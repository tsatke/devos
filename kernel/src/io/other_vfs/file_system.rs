use crate::io::other_vfs::{FsId, VfsError};
use crate::io::path::Path;
use alloc::string::String;
use alloc::vec::Vec;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct VfsHandle(u64);

impl VfsHandle {
    pub fn new(v: u64) -> Self {
        Self(v)
    }
}

pub trait FileSystem: Send + Sync {
    fn fsid(&self) -> FsId;

    fn open(&mut self, path: &Path) -> Result<VfsHandle, VfsError>;

    fn close(&mut self, handle: VfsHandle) -> Result<(), VfsError>;

    fn read_dir(&mut self, path: &Path) -> Result<Vec<String>, VfsError>;

    fn read(&mut self, handle: VfsHandle, buf: &mut [u8], offset: usize)
        -> Result<usize, VfsError>;

    fn write(&mut self, handle: VfsHandle, buf: &[u8], offset: usize) -> Result<usize, VfsError>;

    fn create(&mut self, path: &Path, ftype: FileType) -> Result<(), VfsError>;

    fn remove(&mut self, path: &Path) -> Result<(), VfsError>;
}

pub enum FileType {
    File,
    Directory,
}
