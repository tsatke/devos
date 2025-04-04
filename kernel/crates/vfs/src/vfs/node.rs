use crate::fs::{FileSystem, FsHandle};
use crate::path::OwnedPath;
use crate::{ReadError, Vfs};
use alloc::sync::{Arc, Weak};
use spin::RwLock;

pub struct VfsNode<'vfs> {
    vfs: &'vfs Vfs,
    path: OwnedPath,
    fs_handle: FsHandle,
    fs: Weak<RwLock<dyn FileSystem>>,
}

impl VfsNode<'_> {
    fn fs(&self) -> Option<Arc<RwLock<dyn FileSystem>>> {
        self.fs.upgrade()
    }

    fn handle(&self) -> FsHandle {
        self.fs_handle
    }

    pub fn read<B>(&self, buf: B) -> Result<usize, ReadError>
    where
        B: AsMut<[u8]>,
    {
        todo!()
    }
}
