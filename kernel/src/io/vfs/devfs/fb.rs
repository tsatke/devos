use alloc::boxed::Box;
use alloc::vec::Vec;

use x86_64::structures::paging::{PageSize, PhysFrame, Size4KiB};
use x86_64::PhysAddr;

use crate::driver::pci::{
    BaseAddressRegister, DisplaySubClass, PciDeviceClass, PciStandardHeaderDevice,
};
use kernel_api::syscall::{FileMode, Stat};

use crate::io::vfs::devfs::DevFile;
use crate::io::vfs::{Result, VfsError};

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

    fn stat(&self, stat: &mut Stat) -> Result<()> {
        // TODO: ino, dev, nlink, uid, gid, rdev, blksize, blocks

        stat.mode |= FileMode::S_IFCHR;
        stat.nlink = 1; // TODO: can this change?
        stat.size = self.frames.iter().map(|f| f.size()).sum::<u64>(); // TODO: is this correct? might the memory be shorter?
        stat.blksize = Size4KiB::SIZE; // the size of a PhysFrame
        stat.blocks = self.frames.len() as u64;

        Ok(())
    }

    fn physical_memory(&self) -> Result<Option<Box<dyn Iterator<Item = PhysFrame> + '_>>> {
        Ok(Some(Box::new(self.frames.iter().cloned())))
    }
}

pub fn find_fbs() -> impl Iterator<Item = Fb> {
    find_vga_fbs()
}

fn find_vga_fbs() -> impl Iterator<Item = Fb> {
    crate::driver::pci::devices()
        .find(|device| {
            matches!(
                device.class(),
                PciDeviceClass::DisplayController(DisplaySubClass::VGACompatibleController)
            )
        })
        .map(|device| PciStandardHeaderDevice::new(device.clone()).unwrap())
        .map(|ctrl| {
            let bar0 = ctrl.bar0();
            let (addr, size) = match bar0 {
                BaseAddressRegister::MemorySpace32(bar) => (bar.addr as u64, bar.size),
                BaseAddressRegister::MemorySpace64(bar) => (bar.addr, bar.size),
                _ => {
                    panic!("VGA controller has no memory space BAR")
                }
            };

            (addr..addr + size as u64)
                .step_by(4096)
                .map(PhysAddr::new)
                .map(PhysFrame::<Size4KiB>::containing_address)
                .collect::<Vec<_>>()
        })
        .map(|frames| Fb { frames })
        .into_iter()
}
