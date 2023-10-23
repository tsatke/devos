use crate::io::other_vfs::{FileSystem, FileType, FsId, VfsError, VfsHandle};
use crate::io::path::Path;
use alloc::string::String;
use alloc::vec::Vec;

pub struct VirtualExt2Fs {
    fsid: FsId,
}

impl VirtualExt2Fs {}

impl FileSystem for VirtualExt2Fs {
    fn fsid(&self) -> FsId {
        self.fsid
    }

    fn open(&mut self, path: &Path) -> Result<VfsHandle, VfsError> {
        todo!()
    }

    fn close(&mut self, handle: VfsHandle) -> Result<(), VfsError> {
        todo!()
    }

    fn read_dir(&mut self, path: &Path) -> Result<Vec<String>, VfsError> {
        todo!()
    }

    fn read(
        &mut self,
        handle: VfsHandle,
        buf: &mut [u8],
        offset: usize,
    ) -> Result<usize, VfsError> {
        todo!()
    }

    fn write(&mut self, handle: VfsHandle, buf: &[u8], offset: usize) -> Result<usize, VfsError> {
        todo!()
    }

    fn create(&mut self, path: &Path, ftype: FileType) -> Result<(), VfsError> {
        todo!()
    }

    fn remove(&mut self, path: &Path) -> Result<(), VfsError> {
        todo!()
    }
}
