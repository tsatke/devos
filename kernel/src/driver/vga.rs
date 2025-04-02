use crate::driver::pci::device::PciDevice;
use crate::driver::pci::{PciDriverDescriptor, PciDriverType, PCI_DRIVERS};
use alloc::boxed::Box;
use alloc::vec::Vec;
use conquer_once::spin::OnceCell;
use linkme::distributed_slice;
use spin::Mutex;
use thiserror::Error;
use x86_64::structures::paging::frame::PhysFrameRangeInclusive;
use x86_64::structures::paging::PhysFrame;
use x86_64::PhysAddr;

#[distributed_slice(PCI_DRIVERS)]
static GENERIC_VGA: PciDriverDescriptor = PciDriverDescriptor {
    name: "Generic VGA",
    typ: PciDriverType::Generic,
    probe: |device| device.class() == 0x03 && device.subclass() == 0x00 && device.prog() == 0x00,
    init: |device| {
        let vga_dev = VgaDevice::try_from(device).map_err(Box::new)?;
        vga_devices().lock().push(vga_dev);
        Ok(())
    },
};

static VGA_DEVICES: OnceCell<Mutex<Vec<VgaDevice>>> = OnceCell::uninit();

pub fn vga_devices() -> &'static Mutex<Vec<VgaDevice>> {
    VGA_DEVICES.get_or_init(|| Mutex::new(Vec::new()))
}

#[derive(Debug)]
pub struct VgaDevice {
    _device: PciDevice,
    physical_memory: PhysFrameRangeInclusive,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Error)]
pub enum TryFromPciDeviceError {
    #[error("not a VGA device")]
    NotVga,
    #[error("not a VGA device with memory mapped I/O")]
    NoMemoryBAR,
}

impl TryFrom<PciDevice> for VgaDevice {
    type Error = TryFromPciDeviceError;

    fn try_from(device: PciDevice) -> Result<Self, Self::Error> {
        if device.base_addresses()[0].is_io() {
            return Err(TryFromPciDeviceError::NoMemoryBAR);
        }

        let mut device = device;
        let addr = device.base_addresses()[0].addr(Some(&device.base_addresses()[1])) as u64;
        let size = device.base_addresses_mut()[0].size() as u64;

        let start = PhysFrame::containing_address(PhysAddr::new(addr));
        let end = PhysFrame::containing_address(PhysAddr::new(addr + size - 1));

        Ok(Self {
            _device: device,
            physical_memory: PhysFrameRangeInclusive { start, end },
        })
    }
}

impl VgaDevice {
    #[must_use]
    pub fn physical_memory(&self) -> &PhysFrameRangeInclusive {
        &self.physical_memory
    }
}
