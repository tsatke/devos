use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::ops::BitAnd;

use derive_more::{Constructor, Display};
use x86_64::structures::paging::PhysFrame;

use kernel_api::syscall::{FileMode, Stat};

use crate::io::path::Path;
use crate::io::vfs::error::Result;
use crate::io::vfs::FsId;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct VfsHandle(u64);

impl VfsHandle {
    pub fn new(v: u64) -> Self {
        Self(v)
    }
}

#[derive(Constructor, Debug, Clone, Eq, PartialEq)]
pub struct DirEntry {
    pub name: String,
    pub typ: FileType,
}

pub trait FileSystem: Send + Sync {
    /// Returns the file system id of this file system.
    fn fsid(&self) -> FsId;

    fn exists(&mut self, path: &Path) -> Result<bool> {
        let _ = self.open(path)?;
        Ok(true)
    }

    /// Opens the file at the given path and returns a handle to it.
    /// the handle is file system specific, and the file system must coordinate
    /// and associate it with the appropriate file.
    /// Files that are read from or written to must be opened first with this
    /// method.
    /// Files that have been closed must not be read from or written to.
    /// Implementations should return [`VfsError::HandleClosed`] if the handle
    /// is invalid.
    fn open(&mut self, path: &Path) -> Result<VfsHandle>;

    /// Closes the file associated with the given handle.
    fn close(&mut self, handle: VfsHandle) -> Result<()>;

    fn read_dir(&mut self, path: &Path) -> Result<Vec<DirEntry>>;

    /// Reads from the given offset from the file associated with the given handle
    /// into the given buffer.
    /// This returns how many bytes were read.
    /// If an error occurs, the buffer may be partially filled.
    fn read(&mut self, handle: VfsHandle, buf: &mut [u8], offset: usize) -> Result<usize>;

    /// Writes the given buffer to the given offset from the file associated with
    /// the given handle.
    /// This returns how many bytes were written.
    /// If an error occurs, the file may be partially written.
    fn write(&mut self, handle: VfsHandle, buf: &[u8], offset: usize) -> Result<usize>;

    fn truncate(&mut self, handle: VfsHandle, size: usize) -> Result<()>;

    fn stat(&mut self, handle: VfsHandle, stat: &mut Stat) -> Result<()>;

    fn stat_path(&mut self, p: &Path, stat: &mut Stat) -> Result<()> {
        let handle = self.open(p)?;
        let res = self.stat(handle, stat);
        self.close(handle)?;
        res
    }

    /// Creates a node at the given path.
    /// The type of the node is specified by the [`ftype`] parameter.
    /// The node must be opened with [`FileSystem::open`] to use it.
    ///
    /// In a single threaded environment, if this function returns successfully,
    /// it is guaranteed that [`FileSystem::open`] will succeed with the newly
    /// created node.
    fn create(&mut self, path: &Path, ftype: FileType) -> Result<()>;

    /// Removes the node at the given path.
    fn remove(&mut self, path: &Path) -> Result<()>;

    /// Some files may not be backed by disk storage or virtual memory, but by physical memory.
    /// This method allows the file system to expose the physical memory backing the file in case
    /// it needs to be mmapped (such as frame buffers or other device-specific memory).
    fn physical_memory(
        &self,
        _handle: VfsHandle,
    ) -> Result<Option<Box<dyn Iterator<Item = PhysFrame> + '_>>> {
        Ok(None)
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Display, Debug, Copy, Clone, Eq, PartialEq)]
pub enum FileType {
    RegularFile,
    Directory,
    CharacterDevice,
    BlockDevice,
    FIFO,
    Socket,
    SymbolicLink,
}

impl From<FileMode> for FileType {
    fn from(value: FileMode) -> Self {
        match value.bitand(FileMode::S_IFMT) {
            v if v == FileMode::S_IFREG => Self::RegularFile,
            v if v == FileMode::S_IFDIR => Self::Directory,
            v if v == FileMode::S_IFCHR => Self::CharacterDevice,
            v if v == FileMode::S_IFBLK => Self::BlockDevice,
            v if v == FileMode::S_IFIFO => Self::FIFO,
            v if v == FileMode::S_IFSOCK => Self::Socket,
            v if v == FileMode::S_IFLNK => Self::SymbolicLink,
            _ => unreachable!(),
        }
    }
}
