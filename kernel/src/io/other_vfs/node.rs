use crate::io::other_vfs::file_system::VfsHandle;
use crate::io::other_vfs::FsId;
use crate::io::path::{OwnedPath, Path};

pub struct VfsNode {
    /// The path of this node.
    path: OwnedPath,
    /// The file system specific handle.
    handle: VfsHandle,
    /// The file system id. Can be used to associate the [`handle`] with the
    /// appropriate file system.
    fsid: FsId,
}

impl VfsNode {
    pub(in crate::io::other_vfs) fn new(path: OwnedPath, handle: VfsHandle, fsid: FsId) -> Self {
        Self { path, handle, fsid }
    }
}

impl VfsNode {
    pub fn path(&self) -> &Path {
        self.path.as_path()
    }

    pub fn handle(&self) -> VfsHandle {
        self.handle
    }
}
