use alloc::sync::Arc;

use ext2::{Inode, InodeAddress, Type};
use filesystem::BlockDevice;
use spin::RwLock;

use kernel_api::syscall::{FileMode, Stat};

use crate::io::vfs::error::Result;
use crate::io::vfs::VfsError;

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

    pub fn stat(&self, stat: &mut Stat) -> Result<()> {
        stat.ino = self.inode_num.get() as u64;
        stat.size = self.inner.as_ref().len() as u64;
        
        let typ = self.inner.as_ref().typ();
        stat.mode |= match typ {
            t if t == Type::FIFO => FileMode::S_IFIFO,
            t if t == Type::CharacterDevice => FileMode::S_IFCHR,
            t if t == Type::Directory => FileMode::S_IFDIR,
            t if t == Type::BlockDevice => FileMode::S_IFBLK,
            t if t == Type::RegularFile => FileMode::S_IFREG,
            t if t == Type::SymLink => FileMode::S_IFLNK,
            t if t == Type::UnixSocket => FileMode::S_IFSOCK,
            _ => FileMode::empty(),
        };

        Ok(())
    }
}
