use kernel_api::syscall::{FileMode, Stat};

use crate::io::vfs::devfs::DevFile;
use crate::io::vfs::error::Result;

pub struct Zero;

impl DevFile for Zero {
    fn read(&self, buf: &mut [u8], _: usize) -> Result<usize> {
        buf.fill(0);
        Ok(buf.len())
    }

    fn write(&mut self, buf: &[u8], _: usize) -> Result<usize> {
        Ok(buf.len())
    }

    fn stat(&self, stat: &mut Stat) -> Result<()> {
        // TODO: ino, dev, nlink, uid, gid, rdev

        stat.mode |= FileMode::S_IFCHR; // TODO: permissions
        stat.nlink = 1; // TODO: can this change?
        stat.size = 0;
        stat.blksize = 0;
        stat.blocks = 0;

        Ok(())
    }
}
