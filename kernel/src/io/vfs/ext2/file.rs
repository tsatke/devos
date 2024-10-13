use alloc::sync::Arc;

use ext2::{Inode, InodeAddress, Permissions, Type};
use filesystem::BlockDevice;
use spin::RwLock;

use kernel_api::syscall::{FileMode, Stat};

use crate::io::vfs::error::Result;
use crate::io::vfs::{FsId, VfsError};

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
    pub fn new(
        fsid: FsId,
        fs: Arc<RwLock<ext2::Ext2Fs<T>>>,
        inode_num: InodeAddress,
        inode: Inode,
    ) -> Self {
        let inner = match inode.typ() {
            Type::RegularFile => Inner::RegularFile((inode_num, inode).try_into().unwrap()),
            Type::Directory => Inner::Directory((inode_num, inode).try_into().unwrap()),
            _ => panic!("unsupported inode type"),
        };
        Self {
            fsid,
            fs,
            inode_num,
            inner,
        }
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
            Inner::RegularFile(f) => self
                .fs
                .read()
                .read_from_file(f, offset, buf)
                .map_err(|_| VfsError::ReadError),
            _ => Err(VfsError::Unsupported),
        }
    }

    pub fn write(&mut self, buf: &[u8], offset: usize) -> Result<usize> {
        match &mut self.inner {
            Inner::RegularFile(f) => self
                .fs
                .write()
                .write_to_file(f, offset, buf)
                .map_err(|_| VfsError::WriteError),
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
            _ => unreachable!(),
        };
        stat.mode |= ext2_permissions_to_file_mode(inode.perm());

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

fn ext2_permissions_to_file_mode(permissions: Permissions) -> FileMode {
    let mut mode = FileMode::empty();

    mode |= if permissions.contains(Permissions::UserRead) {
        FileMode::S_IRUSR
    } else {
        FileMode::empty()
    } | if permissions.contains(Permissions::UserWrite) {
        FileMode::S_IWUSR
    } else {
        FileMode::empty()
    } | if permissions.contains(Permissions::UserExec) {
        FileMode::S_IXUSR
    } else {
        FileMode::empty()
    } | if permissions.contains(Permissions::GroupRead) {
        FileMode::S_IRGRP
    } else {
        FileMode::empty()
    } | if permissions.contains(Permissions::GroupWrite) {
        FileMode::S_IWGRP
    } else {
        FileMode::empty()
    } | if permissions.contains(Permissions::GroupExec) {
        FileMode::S_IXGRP
    } else {
        FileMode::empty()
    } | if permissions.contains(Permissions::OtherRead) {
        FileMode::S_IROTH
    } else {
        FileMode::empty()
    } | if permissions.contains(Permissions::OtherWrite) {
        FileMode::S_IWOTH
    } else {
        FileMode::empty()
    } | if permissions.contains(Permissions::OtherExec) {
        FileMode::S_IXOTH
    } else {
        FileMode::empty()
    } | if permissions.contains(Permissions::SetUID) {
        FileMode::S_ISUID
    } else {
        FileMode::empty()
    } | if permissions.contains(Permissions::SetGID) {
        FileMode::S_ISGID
    } else {
        FileMode::empty()
    } | if permissions.contains(Permissions::Sticky) {
        FileMode::S_ISVTX
    } else {
        FileMode::empty()
    };

    mode
}
