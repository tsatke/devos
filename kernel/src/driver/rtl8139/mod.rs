use crate::arch::idt::{end_of_interrupt, InterruptIndex};
use crate::driver::pci::{PciDevice, PciDriverDescriptor, PCI_DRIVERS};
use crate::mem::virt::{AllocationStrategy, MapAt};
use crate::net;
use crate::process::vmm;
use alloc::boxed::Box;
use alloc::format;
use alloc::sync::{Arc, Weak};
use core::error::Error;
use core::hint::spin_loop;
use crossbeam::epoch::Pointable;
use crossbeam::queue::SegQueue;
use foundation::falloc::vec::FVec;
use foundation::future::queue::AsyncBoundedQueue;
use foundation::net::MacAddr;
use linkme::distributed_slice;
use log::{error, info, trace};
use netstack::device::RawDataLinkFrame;
use netstack::interface::Interface;
use spin::Mutex;
use thiserror::Error;
use x86_64::instructions::port::Port;
use x86_64::structures::idt::InterruptStackFrame;
use x86_64::structures::paging::{PageTableFlags, PhysFrame};
use x86_64::PhysAddr;

#[distributed_slice(PCI_DRIVERS)]
static RTL8239_DRIVER: PciDriverDescriptor = PciDriverDescriptor {
    name: "RTL8139",
    probe: Rtl8139::probe,
    init: Rtl8139::init,
};

static RTL8139_CARDS: SegQueue<Rtl8139> = SegQueue::new();

pub extern "x86-interrupt" fn rtl8139_interrupt_handler(_stack_frame: InterruptStackFrame) {
    let len = RTL8139_CARDS.len();
    trace!("servicing {len} RTL8139 cards");
    for _ in 0..len {
        if let Some(rtl) = RTL8139_CARDS.pop() {
            match rtl.interrupt_received() {
                Ok(_) => RTL8139_CARDS.push(rtl),
                Err(InterruptRoutineError::DeviceDisconnected) => {
                    info!("RTL8139 device disconnected");
                }
            }
        }
    }
    unsafe { end_of_interrupt() };
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Error)]
pub enum InterruptRoutineError {
    #[error("device is not connected any more")]
    DeviceDisconnected,
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

pub struct Rtl8139 {
    mac_addr: MacAddr,
    pci_device: Weak<Mutex<PciDevice>>,
    registers: Mutex<Registers>,
    rx_queue: Arc<AsyncBoundedQueue<RawDataLinkFrame>>,
    tx_queue: Arc<AsyncBoundedQueue<RawDataLinkFrame>>,
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

            // FIXME: this must probably be located in the kernel heap, since the interrupts can happen in any process, and we don't switch address spaces
            let virt_addr = vmm()
                .allocate_memory_backed_vmobject(
                    format!("rtl8139 {guard} bar{i}"),
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

        guard.interrupt_line.write(InterruptIndex::Rtl8139.as_u8());

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
            rx_queue: Arc::new(AsyncBoundedQueue::new(16)),
            tx_queue: Arc::new(AsyncBoundedQueue::new(16)),
        })
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

        let nic = Interface::new(rtl.mac_addr, rtl.rx_queue.clone(), rtl.tx_queue.clone());
        net::register_nic(nic)?;

        RTL8139_CARDS.push(rtl);
        Ok(())
    }

    fn interrupt_received(&self) -> Result<(), InterruptRoutineError> {
        todo!()
    }
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
