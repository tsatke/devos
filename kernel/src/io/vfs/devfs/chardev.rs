use crate::io::vfs::{CharacterDeviceFile, InodeBase, InodeNum, ReadError, Stat, WriteError};
use alloc::string::String;

pub struct CharDev<R, W> {
    fsid: u64,
    num: InodeNum,
    name: String,
    read: R,
    write: W,
}

impl<R, W> CharDev<R, W> {
    pub fn new(fsid: u64, num: InodeNum, name: String, read: R, write: W) -> Self {
        Self {
            fsid,
            num,
            name,
            read,
            write,
        }
    }
}

impl<R, W> InodeBase for CharDev<R, W>
where
    R: Fn(&mut dyn AsMut<[u8]>) -> Result<usize, ReadError> + Sync + Send,
    W: FnMut(&dyn AsRef<[u8]>) -> Result<usize, WriteError> + Sync + Send,
{
    fn name(&self) -> String {
        self.name.clone()
    }

    fn stat(&self) -> Stat {
        Stat {
            dev: self.fsid,
            inode: self.num,
            ..Default::default()
        }
    }
}

impl<R, W> CharacterDeviceFile for CharDev<R, W>
where
    R: Fn(&mut dyn AsMut<[u8]>) -> Result<usize, ReadError> + Sync + Send,
    W: FnMut(&dyn AsRef<[u8]>) -> Result<usize, WriteError> + Sync + Send,
{
    fn read(&self, buf: &mut dyn AsMut<[u8]>) -> Result<usize, ReadError> {
        (self.read)(buf)
    }

    fn write(&mut self, buf: &dyn AsRef<[u8]>) -> Result<usize, WriteError> {
        (self.write)(buf)
    }
}
