use crate::driver::hpet::Inner;
use crate::driver::pci::{PciDevice, PciDriverDescriptor, PCI_DRIVERS};
use crate::mem::virt::{AllocationStrategy, MapAt};
use crate::net;
use crate::process::vmm;
use crate::time::HpetInstantProvider;
use alloc::boxed::Box;
use alloc::format;
use alloc::string::ToString;
use alloc::sync::{Arc, Weak};
use conquer_once::spin::OnceCell;
use core::error::Error;
use core::future::Future;
use core::hint::spin_loop;
use core::pin::Pin;
use core::ptr::NonNull;
use core::task::{Context, Poll, Waker};
use core::time::Duration;
use crossbeam::queue::ArrayQueue;
use foundation::net::MacAddr;
use foundation::time::Instant;
use futures::future::BoxFuture;
use futures::FutureExt;
use linkme::distributed_slice;
use log::{debug, info, trace};
use netstack::device::{Device, RawDataLinkFrame};
use spin::Mutex;
use thiserror::Error;
use volatile::{VolatileFieldAccess, VolatilePtr};
use x86_64::instructions::hlt;
use x86_64::instructions::port::Port;
use x86_64::structures::paging::{PageTableFlags, PhysFrame};
use x86_64::PhysAddr;

#[distributed_slice(PCI_DRIVERS)]
static RTL8239_DRIVER: PciDriverDescriptor = PciDriverDescriptor {
    name: "RTL8139",
    probe: Rtl8139::probe,
    init: Rtl8139::init,
};

static RTL_WAKERS: OnceCell<ArrayQueue<Waker>> = OnceCell::uninit();

fn rtl_wakers() -> &'static ArrayQueue<Waker> {
    RTL_WAKERS.get_or_init(|| ArrayQueue::new(16)) // should be plenty
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Error)]
pub enum TryFromPciDeviceError {
    #[error("device is not a RTL8139")]
    NotRtl8139,
    #[error("device has no IO base address register")]
    NoIoBaseAddressRegister,
    #[error("device has invalid base address register")]
    InvalidBarAddress,
    #[error("device is not connected")]
    DeviceDisconnected,
    #[error("failed to allocate memory")]
    AllocError,
}

impl TryFrom<Weak<Mutex<PciDevice>>> for Rtl8139 {
    type Error = TryFromPciDeviceError;

    fn try_from(device: Weak<Mutex<PciDevice>>) -> Result<Self, Self::Error> {
        let device = device
            .upgrade()
            .ok_or(TryFromPciDeviceError::DeviceDisconnected)?;

        let mut guard = device.lock();
        if !Rtl8139::probe(&guard) {
            return Err(TryFromPciDeviceError::NotRtl8139);
        }

        guard.enable_bus_mastering();

        let (iobase, size) = {
            let iobar = guard
                .base_addresses
                .iter_mut()
                .find(|bar| bar.is_io())
                .ok_or(TryFromPciDeviceError::NoIoBaseAddressRegister)?;
            (iobar.addr(None), iobar.size())
        };
        trace!(
            "RTL8139 IO base address/size: {:#x} / 0x{:02x}",
            iobase,
            size
        );

        // map memory bars
        for i in 0..guard.base_addresses.len() {
            if !guard.base_addresses[i].exists() {
                continue;
            }

            let size = guard.base_addresses[i].size();

            let next = guard.base_addresses.iter().nth(i + 1);
            let bar = &guard.base_addresses[i];
            let phys_addr = PhysAddr::try_new(bar.addr(next) as u64)
                .map_err(|_| TryFromPciDeviceError::InvalidBarAddress)?;

            let virt_addr = vmm()
                .allocate_memory_backed_vmobject(
                    format!("rtl8139 bar{i}"),
                    MapAt::Anywhere,
                    size,
                    AllocationStrategy::MapNow(&[PhysFrame::containing_address(phys_addr)]),
                    PageTableFlags::PRESENT
                        | PageTableFlags::NO_EXECUTE
                        | PageTableFlags::WRITABLE
                        | PageTableFlags::NO_CACHE,
                )
                .map_err(|_| TryFromPciDeviceError::AllocError)?;
            trace!("mapped RTL8139 BAR{i} at {virt_addr:p} -> 0x{phys_addr:02x}",);
        }

        let mut registers =
            Registers::new(u16::try_from(iobase).expect("io base offset should fit into a u16"));

        // turn on
        unsafe { registers.config_1.write(0x0) };

        // software reset
        unsafe {
            registers.cmd.write(0x10);
            while registers.cmd.read() & 0x10 > 0 {
                spin_loop();
            }
        }

        let mac_addr = unsafe {
            let lo = registers.mac0_5_lo.read();
            let hi = registers.mac0_5_hi.read();
            MacAddr::new([
                lo as u8,
                (lo >> 8) as u8,
                (lo >> 16) as u8,
                (lo >> 24) as u8,
                hi as u8,
                (hi >> 8) as u8,
            ])
        };

        Ok(Self {
            mac_addr,
            pci_device: Arc::downgrade(&device),
            registers: Mutex::new(registers),
        })
    }
}

pub struct Rtl8139 {
    mac_addr: MacAddr,
    pci_device: Weak<Mutex<PciDevice>>,
    registers: Mutex<Registers>,
}

struct Registers {
    mac0_5_lo: Port<u32>,
    mac0_5_hi: Port<u16>,
    mar0_7_lo: Port<u32>,
    mar0_7_hi: Port<u16>,
    rbstart: Port<u32>,
    cmd: Port<u8>,
    imr: Port<u16>,
    isr: Port<u16>,
    config_1: Port<u8>,
}

impl Registers {
    fn new(iobase: u16) -> Self {
        Self {
            mac0_5_lo: Port::new(iobase + 0x00),
            mac0_5_hi: Port::new(iobase + 0x04),
            mar0_7_lo: Port::new(iobase + 0x08),
            mar0_7_hi: Port::new(iobase + 0x0C),
            rbstart: Port::new(iobase + 0x30),
            cmd: Port::new(iobase + 0x37),
            imr: Port::new(iobase + 0x3C),
            isr: Port::new(iobase + 0x3E),
            config_1: Port::new(iobase + 0x52),
        }
    }
}

impl Rtl8139 {
    pub const VENDOR_ID: u16 = 0x10EC;
    pub const DEVICE_ID: u16 = 0x8139;

    pub fn probe(device: &PciDevice) -> bool {
        device.vendor_id == Self::VENDOR_ID && device.device_id == Self::DEVICE_ID
    }

    pub fn init(device: Weak<Mutex<PciDevice>>) -> Result<(), Box<dyn Error>> {
        let rtl = Self::try_from(device)?;
        info!("RTL8139 MAC address: {}", rtl.mac_addr);

        net::register_nic(Box::new(rtl))?;
        Ok(())
    }
}

impl Device for Rtl8139 {
    fn mac_address(&self) -> MacAddr {
        self.mac_addr
    }

    fn read_frame(&self) -> BoxFuture<RawDataLinkFrame> {
        ReadFrameFuture {
            rtl: self.pci_device.clone(),
        }
        .boxed()
    }

    fn write_frame(&self, _frame: RawDataLinkFrame) -> BoxFuture<()> {
        todo!()
    }
}

pub struct ReadFrameFuture {
    rtl: Weak<Mutex<PciDevice>>,
}

impl Future for ReadFrameFuture {
    type Output = RawDataLinkFrame;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let rtl = self.rtl.upgrade().expect("device should be connected");
        let _rtl = if let Some(rtl) = rtl.try_lock() {
            rtl
        } else {
            cx.waker().wake_by_ref();
            return Poll::Pending;
        };

        todo!("check whether there's a frame to read, otherwise register waker in RTL_WAKERS");
    }
}
