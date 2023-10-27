use crate::io::vfs::devfs::DevFile;
use crate::io::vfs::VfsError;

pub struct Zero;

impl DevFile for Zero {
    fn read(&self, buf: &mut [u8], _: usize) -> Result<usize, VfsError> {
        buf.fill(0);
        Ok(buf.len())
    }

    fn write(&mut self, buf: &[u8], _: usize) -> Result<usize, VfsError> {
        Ok(buf.len())
    }
}
