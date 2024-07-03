use alloc::boxed::Box;
use alloc::vec::Vec;

use x86_64::PhysAddr;
use x86_64::structures::paging::{PhysFrame, Size4KiB};

use pci::{BaseAddressRegister, PciStandardHeaderDevice};

use crate::io::vfs::{Result, VfsError};
use crate::io::vfs::devfs::DevFile;

#[derive(Debug, Clone)]
pub struct Fb {
    frames: Vec<PhysFrame>,
}

impl DevFile for Fb {
    fn read(&self, _: &mut [u8], _: usize) -> Result<usize> {
        Err(VfsError::Unsupported)
    }

    fn write(&mut self, _: &[u8], _: usize) -> Result<usize> {
        Err(VfsError::Unsupported)
    }

    fn physical_memory(&self) -> Result<Option<Box<dyn Iterator<Item=PhysFrame> + '_>>> {
        Ok(Some(Box::new(self.frames.iter().cloned())))
    }
}

pub fn find_fbs() -> impl Iterator<Item=Fb> {
    find_vga_fbs()
}

fn find_vga_fbs() -> impl Iterator<Item=Fb> {
    pci::devices()
        .find(|device| {
            matches!(
                device.class(),
                pci::PciDeviceClass::DisplayController(
                    pci::DisplaySubClass::VGACompatibleController
                )
            )
        })
        .map(|device| PciStandardHeaderDevice::new(device.clone()).unwrap())
        .and_then(|ctrl| {
            let bar0 = ctrl.bar0();
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
                .collect::<Vec<_>>();
            Some(frames)
        })
        .map(|frames| Fb { frames })
        .into_iter()
}