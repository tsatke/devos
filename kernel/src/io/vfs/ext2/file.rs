use alloc::sync::Arc;

use ext2::{Inode, InodeAddress, Type};
use filesystem::BlockDevice;
use spin::RwLock;

use kernel_api::syscall::{FileMode, Stat};

use crate::io::vfs::{FsId, VfsError};
use crate::io::vfs::error::Result;

pub struct Ext2Inode<T> {
    fsid: FsId,
    fs: Arc<RwLock<ext2::Ext2Fs<T>>>,
    inode_num: InodeAddress,
    inner: Inner,
}

impl<T> Ext2Inode<T>
where
    T: BlockDevice,
{
    pub fn new(fsid: FsId, fs: Arc<RwLock<ext2::Ext2Fs<T>>>, inode_num: InodeAddress, inode: Inode) -> Self {
        let inner = match inode.typ() {
            Type::RegularFile => Inner::RegularFile((inode_num, inode).try_into().unwrap()),
            Type::Directory => Inner::Directory((inode_num, inode).try_into().unwrap()),
            _ => panic!("unsupported inode type"),
        };
        Self { fsid, fs, inode_num, inner }
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
        let inode = self.inner.as_ref();

        stat.dev = self.fsid.0;
        stat.ino = self.inode_num.get() as u64;

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

        stat.nlink = inode.num_hard_links() as u32;
        // TODO: uid, gid
        stat.rdev = 0; // TODO: set correct rdev
        stat.size = inode.len() as u64;
        stat.atime = inode.last_access_time().into();
        stat.mtime = inode.last_modification_time().into();
        stat.ctime = inode.creation_time().into();
        stat.blksize = self.fs.read().superblock().block_size() as u64;
        stat.blocks = inode.num_disk_sectors() as u64;

        Ok(())
    }
}
