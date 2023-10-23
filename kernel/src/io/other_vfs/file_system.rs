use crate::io::other_vfs::{FsId, VfsError};
use crate::io::path::{OwnedPath, Path};
use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct VfsHandle(u64);

impl VfsHandle {
    pub fn new(v: u64) -> Self {
        Self(v)
    }
}

pub trait FileSystem: Send + Sync {
    /// Returns the file system id of this file system.
    fn fsid(&self) -> FsId;

    /// Opens the file at the given path and returns a handle to it.
    /// the handle is file system specific, and the file system must coordinate
    /// and associate it with the appropriate file.
    /// Files that are read from or written to must be opened first with this
    /// method.
    /// Files that have been closed must not be read from or written to.
    /// Implementations should return [`VfsError::HandleClosed`] if the handle
    /// is invalid.
    fn open(&mut self, path: &Path) -> Result<VfsHandle, VfsError>;

    /// Closes the file associated with the given handle.
    fn close(&mut self, handle: VfsHandle) -> Result<(), VfsError>;

    /// Returns all entries in the directory associated with the given path.
    ///
    /// This returns an error if the path is not a directory.
    ///
    /// All returned paths are absolute, but are to be interpreted relative to
    /// the mount point at which this file system is mounted.
    /// As an example, if the file system is mounted at `/foo`, and one of the
    /// returned paths is `/bar`, then the actual path is `/foo/bar`.
    /// Calls to [`FileSystem::open`] will succeed with `/bar`, [`crate::io::other_vfs::Vfs::open`]
    /// will succeed with `/foo/bar`.
    fn read_dir(&mut self, path: &Path) -> Result<Vec<OwnedPath>, VfsError>;

    /// Reads from the given offset from the file associated with the given handle
    /// into the given buffer.
    /// This returns how many bytes were read.
    /// If an error occurs, the buffer may be partially filled.
    fn read(&mut self, handle: VfsHandle, buf: &mut [u8], offset: usize)
        -> Result<usize, VfsError>;

    /// Writes the given buffer to the given offset from the file associated with
    /// the given handle.
    /// This returns how many bytes were written.
    /// If an error occurs, the file may be partially written.
    fn write(&mut self, handle: VfsHandle, buf: &[u8], offset: usize) -> Result<usize, VfsError>;

    /// Creates a node at the given path.
    /// The type of the node is specified by the [`ftype`] parameter.
    /// The node must be opened with [`FileSystem::open`] to use it.
    ///
    /// In a single threaded environment, if this function returns successfully,
    /// it is guaranteed that [`FileSystem::open`] will succeed with the newly
    /// created node.
    fn create(&mut self, path: &Path, ftype: FileType) -> Result<(), VfsError>;

    /// Removes the node at the given path.
    fn remove(&mut self, path: &Path) -> Result<(), VfsError>;
}

pub enum FileType {
    File,
    Directory,
}
