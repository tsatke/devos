use crate::fs::{FileSystem, FsHandle};
use crate::path::OwnedPath;
use crate::Vfs;
use alloc::sync::Weak;
use spin::RwLock;

pub struct VfsNode<'vfs> {
    _vfs: &'vfs Vfs,
    _path: OwnedPath,
    _fs_handle: FsHandle,
    _fs: Weak<RwLock<dyn FileSystem>>,
}
