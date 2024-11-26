use crate::driver::pci::{PciDevice, PciDriverDescriptor, PCI_DRIVERS};
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
use x86_64::structures::paging::{PageSize, PhysFrame, Size4KiB};
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
    _device: Weak<Mutex<PciDevice>>,

    frames: Arc<FVec<PhysFrame>>,
}

impl VgaDevice {
    fn probe(device: &PciDevice) -> bool {
        device.class == 0x03 && device.subclass == 0x00
    }

    fn init(device: Weak<Mutex<PciDevice>>) -> Result<(), Box<dyn Error>> {
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

impl TryFrom<Arc<Mutex<PciDevice>>> for VgaDevice {
    type Error = TryFromPciDeviceError;

    fn try_from(value: Arc<Mutex<PciDevice>>) -> Result<Self, Self::Error> {
        let mut device = value.lock();
        if device.base_addresses[0].is_io() {
            return Err(TryFromPciDeviceError::NoMemorySpaceBar);
        }

        let addr = device.base_addresses[0].addr(Some(&device.base_addresses[1])) as u64;
        let size = device.base_addresses[0].size() as u64;

        let frames = (addr..addr + size)
            .step_by(Size4KiB::SIZE as usize)
            .map(PhysAddr::new_truncate)
            .map(PhysFrame::<Size4KiB>::containing_address)
            .collect::<Vec<_>>(); // TODO: use FVec
        Ok(Self {
            _device: Arc::downgrade(&value),
            frames: Arc::new(frames.into()),
        })
    }
}
