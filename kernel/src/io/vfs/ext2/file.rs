use alloc::sync::Arc;

use ext2::{Inode, InodeAddress, Type};
use filesystem::BlockDevice;
use spin::RwLock;

use crate::io::vfs::{Stat, VfsError};
use crate::io::vfs::error::Result;

pub struct Ext2Inode<T> {
    fs: Arc<RwLock<ext2::Ext2Fs<T>>>,
    inode_num: InodeAddress,
    inner: Inner,
}

impl<T> Ext2Inode<T>
where
    T: BlockDevice,
{
    pub fn new(fs: Arc<RwLock<ext2::Ext2Fs<T>>>, inode_num: InodeAddress, inode: Inode) -> Self {
        let inner = match inode.typ() {
            Type::RegularFile => Inner::RegularFile((inode_num, inode).try_into().unwrap()),
            Type::Directory => Inner::Directory((inode_num, inode).try_into().unwrap()),
            _ => panic!("unsupported inode type"),
        };
        Self { fs, inode_num, inner }
    }
}

enum Inner {
    RegularFile(ext2::RegularFile),
    Directory(ext2::Directory),
}

impl AsRef<Inode> for Inner {
    fn as_ref(&self) -> &Inode {
        match self {
            Inner::RegularFile(f) => f,
            Inner::Directory(d) => d,
        }
    }
}

impl<T> Ext2Inode<T>
where
    T: BlockDevice,
{
    pub fn read(&self, buf: &mut [u8], offset: usize) -> Result<usize> {
        match &self.inner {
            Inner::RegularFile(f) => self.fs.read().read_from_file(f, offset, buf).map_err(|_| VfsError::ReadError),
            _ => Err(VfsError::Unsupported),
        }
    }

    pub fn write(&mut self, buf: &[u8], offset: usize) -> Result<usize> {
        match &mut self.inner {
            Inner::RegularFile(f) => self.fs.write().write_to_file(f, offset, buf).map_err(|_| VfsError::WriteError),
            _ => Err(VfsError::Unsupported),
        }
    }

    pub fn stat(&self) -> Result<Stat> {
        let inode = self.inode_num.get() as u64; // TODO: is this correct?
        let size = self.inner.as_ref().len() as u64;
        Ok(Stat {
            inode,
            size,
            ..Default::default() // TODO: fill in the rest
        })
    }
}
