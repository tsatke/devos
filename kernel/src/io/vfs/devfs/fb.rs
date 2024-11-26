use alloc::boxed::Box;
use x86_64::structures::paging::{PageSize, PhysFrame, Size4KiB};

use crate::driver::vga;
use crate::driver::vga::VgaDevice;
use crate::io::vfs::devfs::DevFile;
use crate::io::vfs::{Result, VfsError};
use kernel_api::syscall::{FileMode, Stat};

pub fn find_fbs() -> impl Iterator<Item = Fb> {
    vga::devices()
        .lock()
        .try_clone()
        .unwrap() // TODO: handle error
        .into_iter()
        .map(Fb::Vga)
}

#[derive(Clone)]
pub enum Fb {
    Vga(VgaDevice),
}

impl Fb {
    fn frames(&self) -> impl Iterator<Item = &PhysFrame> {
        match self {
            Fb::Vga(vga) => vga.physical_frames().iter(),
        }
    }
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
        stat.size = self.frames().map(|f| f.size()).sum::<u64>(); // TODO: is this correct? might the memory be shorter?
        stat.blksize = Size4KiB::SIZE; // the size of a PhysFrame
        stat.blocks = self.frames().count() as u64;

        Ok(())
    }

    fn physical_memory(&self) -> Result<Option<Box<dyn Iterator<Item = PhysFrame> + '_>>> {
        Ok(Some(Box::new(self.frames().cloned())))
    }
}
