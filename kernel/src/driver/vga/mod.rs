use crate::driver::pci::{
    BaseAddressRegister, DisplaySubClass, PciDevice, PciDeviceClass, PciDriverDescriptor,
    PCI_DRIVERS,
};
use alloc::boxed::Box;
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use conquer_once::spin::OnceCell;
use core::alloc::AllocError;
use core::error::Error;
use foundation::falloc::vec::FVec;
use linkme::distributed_slice;
use spin::Mutex;
use thiserror::Error;
use x86_64::structures::paging::{PhysFrame, Size4KiB};
use x86_64::PhysAddr;

#[distributed_slice(PCI_DRIVERS)]
static VGA_DEVICE_DRIVER: PciDriverDescriptor = PciDriverDescriptor {
    name: "VGADevice",
    probe: VgaDevice::probe,
    init: VgaDevice::init,
};

static VGA_DEVICES: OnceCell<Mutex<FVec<VgaDevice>>> = OnceCell::uninit();

fn register_vga_device(device: VgaDevice) -> Result<(), Box<dyn Error>> {
    match devices().lock().try_push(device) {
        Ok(_) => Ok(()),
        Err(_e) => Err(Box::new(AllocError)),
    }
}

pub fn devices() -> &'static Mutex<FVec<VgaDevice>> {
    VGA_DEVICES.get_or_init(Mutex::default)
}

#[derive(Clone)]
pub struct VgaDevice {
    _device: Weak<PciDevice>,

    frames: Arc<FVec<PhysFrame>>,
}

impl VgaDevice {
    fn probe(device: &PciDevice) -> bool {
        matches!(
            device.class(),
            PciDeviceClass::DisplayController(DisplaySubClass::VGACompatibleController)
        )
    }

    fn init(device: Weak<PciDevice>) -> Result<(), Box<dyn Error>> {
        let device = device.upgrade().ok_or(AllocError)?;
        register_vga_device(VgaDevice::try_from(device)?)?;
        Ok(())
    }

    pub fn physical_frames(&self) -> &'_ [PhysFrame] {
        &self.frames
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Error)]
pub enum TryFromPciDeviceError {
    #[error("device is not connected")]
    DeviceDisconnected,
    #[error("device has no memory space BAR")]
    NoMemorySpaceBar,
    #[error("out of memory")]
    AllocError,
}

impl TryFrom<Arc<PciDevice>> for VgaDevice {
    type Error = TryFromPciDeviceError;

    fn try_from(value: Arc<PciDevice>) -> Result<Self, Self::Error> {
        let bar0 = value.bar0();
        let (addr, size) = match bar0 {
            BaseAddressRegister::MemorySpace32(bar) => (bar.addr as u64, bar.size),
            BaseAddressRegister::MemorySpace64(bar) => (bar.addr, bar.size),
            _ => {
                panic!("VGA controller has no memory space BAR")
            }
        };

        let frames = (addr..addr + size as u64)
            .step_by(4096)
            .map(PhysAddr::new)
            .map(PhysFrame::<Size4KiB>::containing_address)
            .collect::<Vec<_>>(); // TODO: use FVec
        Ok(Self {
            _device: Arc::downgrade(&value),
            frames: Arc::new(frames.into()),
        })
    }
}
