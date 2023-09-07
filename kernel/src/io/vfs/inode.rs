use crate::io::path::OwnedPath;
use crate::io::vfs::{CreateError, LookupError, MountError, Permission, ReadError, WriteError};
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::fmt::{Debug, Display, Formatter};
use spin::RwLock;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Default)]
pub struct InodeNum(u64);

impl Display for InodeNum {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<T: Into<u64>> From<T> for InodeNum {
    fn from(v: T) -> Self {
        Self(v.into())
    }
}

pub trait Fs {
    fn root_inode(&self) -> Inode;
}

pub trait InodeBase: Send + Sync {
    fn num(&self) -> InodeNum;

    fn name(&self) -> String;

    fn stat(&self) -> Stat;
}

#[derive(Copy, Clone, Default)]
pub struct Stat {
    pub dev: u64,
    pub inode: InodeNum,
    pub rdev: u32,
    pub nlink: u32,
    pub uid: u32,
    pub gid: u32,
    pub size: u64,
    pub atime: u32,
    pub mtime: u32,
    pub ctime: u32,
    pub blksize: u32,
    pub blocks: u32,
}

pub type BlockDeviceHandle = Arc<RwLock<dyn BlockDeviceFile>>;
pub type CharacterDeviceHandle = Arc<RwLock<dyn CharacterDeviceFile>>;
pub type DirHandle = Arc<RwLock<dyn Dir>>;
pub type FileHandle = Arc<RwLock<dyn File>>;
pub type SymlinkHandle = Arc<RwLock<dyn Symlink>>;

#[derive(Clone)]
pub enum Inode {
    BlockDevice(BlockDeviceHandle),
    CharacterDevice(CharacterDeviceHandle),
    Dir(DirHandle),
    File(FileHandle),
    Symlink(SymlinkHandle),
}

impl PartialEq for Inode {
    fn eq(&self, other: &Self) -> bool {
        self.num() == other.num() && self.stat().dev == other.stat().dev
    }
}

impl Inode {
    pub fn new_file<F>(f: F) -> Self
    where
        F: 'static + File,
    {
        Self::File(Arc::new(RwLock::new(f)))
    }

    pub fn new_block_device_file<F>(f: F) -> Self
    where
        F: 'static + BlockDeviceFile,
    {
        Self::BlockDevice(Arc::new(RwLock::new(f)))
    }

    pub fn new_character_device_file<F>(f: F) -> Self
    where
        F: 'static + CharacterDeviceFile,
    {
        Self::CharacterDevice(Arc::new(RwLock::new(f)))
    }

    pub fn new_dir<D>(d: D) -> Self
    where
        D: 'static + Dir,
    {
        Self::Dir(Arc::new(RwLock::new(d)))
    }

    pub fn new_symlink<S>(s: S) -> Self
    where
        S: 'static + Symlink,
    {
        Self::Symlink(Arc::new(RwLock::new(s)))
    }

    pub fn as_file(&self) -> Option<FileHandle> {
        match self {
            Inode::File(f) => Some(f.clone()),
            _ => None,
        }
    }

    pub fn as_dir(&self) -> Option<DirHandle> {
        match self {
            Inode::Dir(d) => Some(d.clone()),
            _ => None,
        }
    }

    pub fn as_block_device_file(&self) -> Option<BlockDeviceHandle> {
        match self {
            Inode::BlockDevice(d) => Some(d.clone()),
            _ => None,
        }
    }

    pub fn as_character_device_file(&self) -> Option<CharacterDeviceHandle> {
        match self {
            Inode::CharacterDevice(d) => Some(d.clone()),
            _ => None,
        }
    }

    pub fn as_symlink(&self) -> Option<SymlinkHandle> {
        match self {
            Inode::Symlink(d) => Some(d.clone()),
            _ => None,
        }
    }

    pub fn is_file(&self) -> bool {
        matches!(self, Inode::File(_))
    }

    pub fn is_dir(&self) -> bool {
        matches!(self, Inode::Dir(_))
    }

    pub fn is_block_device_file(&self) -> bool {
        matches!(self, Inode::BlockDevice(_))
    }

    pub fn is_character_device_file(&self) -> bool {
        matches!(self, Inode::CharacterDevice(_))
    }

    pub fn is_symlink(&self) -> bool {
        matches!(self, Inode::Symlink(_))
    }
}

impl Display for Inode {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl Debug for Inode {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("INode")
            .field(
                "type",
                &match self {
                    Inode::File(_) => "File",
                    Inode::Dir(_) => "Dir",
                    Inode::BlockDevice(_) => "BlockDevice",
                    Inode::CharacterDevice(_) => "CharacterDevice",
                    Inode::Symlink(_) => "Symlink",
                },
            )
            .field("device", &self.stat().dev)
            .field("inode_num", &self.num())
            .field("name", &self.name())
            .finish()
    }
}

impl InodeBase for Inode {
    fn num(&self) -> InodeNum {
        match self {
            Inode::File(file) => file.read().num(),
            Inode::Dir(dir) => dir.read().num(),
            Inode::BlockDevice(dev) => dev.read().num(),
            Inode::CharacterDevice(dev) => dev.read().num(),
            Inode::Symlink(symlink) => symlink.read().num(),
        }
    }

    fn name(&self) -> String {
        match self {
            Inode::File(file) => file.read().name(),
            Inode::Dir(dir) => dir.read().name(),
            Inode::BlockDevice(dev) => dev.read().name(),
            Inode::CharacterDevice(dev) => dev.read().name(),
            Inode::Symlink(symlink) => symlink.read().name(),
        }
    }

    fn stat(&self) -> Stat {
        match self {
            Inode::File(file) => file.read().stat(),
            Inode::Dir(dir) => dir.read().stat(),
            Inode::BlockDevice(dev) => dev.read().stat(),
            Inode::CharacterDevice(dev) => dev.read().stat(),
            Inode::Symlink(symlink) => symlink.read().stat(),
        }
    }
}

pub trait BlockDeviceFile: InodeBase {
    fn block_count(&self) -> usize;

    fn block_size(&self) -> usize;

    fn read_block(&self, block: u64, buf: &mut dyn AsMut<[u8]>) -> Result<(), ReadError>;

    fn read_at(&self, offset: u64, buf: &mut dyn AsMut<[u8]>) -> Result<usize, ReadError>;

    fn write_at(&mut self, offset: u64, buf: &dyn AsRef<[u8]>) -> Result<usize, WriteError>;
}

pub trait CharacterDeviceFile: InodeBase {
    fn read_at(&self, offset: u64, buf: &mut dyn AsMut<[u8]>) -> Result<usize, ReadError>;

    fn write_at(&mut self, offset: u64, buf: &dyn AsRef<[u8]>) -> Result<usize, WriteError>;
}

pub trait File: InodeBase {
    fn size(&self) -> u64;

    fn truncate(&mut self, size: u64) -> Result<(), WriteError>;

    fn reserve(&mut self, additional: u64) -> Result<(), WriteError> {
        self.truncate(self.size() + additional)
    }

    fn read_at(&self, offset: u64, buf: &mut dyn AsMut<[u8]>) -> Result<usize, ReadError>;

    fn write_at(&mut self, offset: u64, buf: &dyn AsRef<[u8]>) -> Result<usize, WriteError>;
}

pub enum CreateNodeType {
    File,
    Dir,
}

pub trait Dir: InodeBase {
    /// Searches for an [`Inode`] in the immediate children of this dir by name.
    fn lookup(&self, name: &dyn AsRef<str>) -> Result<Inode, LookupError>;

    /// Creates a new [`Inode`] of the given type. The operation may fail if a type
    /// is not supported.
    fn create(
        &mut self,
        name: &dyn AsRef<str>,
        typ: CreateNodeType,
        permission: Permission,
    ) -> Result<Inode, CreateError>;

    /// Returns a vec of [`INodes`] that are contained within this directory.
    fn children(&self) -> Result<Vec<Inode>, LookupError>;

    /// Adds an (possibly external) [`Inode`] to the children of this dir. External
    /// means, that the given [`Inode`] does not necessarily belong to the same file
    /// system or device as this node.
    fn mount(&mut self, node: Inode) -> Result<(), MountError>;
}

pub trait Symlink: InodeBase {
    fn target(&self) -> Result<String, ReadError>;

    fn target_path(&self) -> Result<OwnedPath, ReadError> {
        Ok(OwnedPath::from(self.target()?))
    }
}
