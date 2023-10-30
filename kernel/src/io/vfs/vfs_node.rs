use crate::io::path::{OwnedPath, Path};
use crate::io::vfs;
use crate::io::vfs::{FileSystem, VfsHandle};
use alloc::sync::Arc;
use spin::RwLock;
use x86_64::instructions::interrupts;

pub struct VfsNode {
    /// The path of this node.
    path: OwnedPath,
    /// The file system specific handle.
    handle: VfsHandle,
    fs: Arc<RwLock<dyn FileSystem>>,
}

impl VfsNode {
    pub(in crate::io::vfs) fn new(
        path: OwnedPath,
        handle: VfsHandle,
        fs: Arc<RwLock<dyn FileSystem>>,
    ) -> Self {
        Self { path, handle, fs }
    }

    pub fn path(&self) -> &Path {
        self.path.as_path()
    }

    pub fn handle(&self) -> VfsHandle {
        self.handle
    }

    pub fn fs(&self) -> &Arc<RwLock<dyn FileSystem>> {
        &self.fs
    }
}

impl Drop for VfsNode {
    fn drop(&mut self) {
        assert!(
            interrupts::are_enabled(),
            "interrupts must be enabled when dropping a vfsnode"
        ); // best effort, there is no way to guarantee that we don't get preempted right after this, so...
        vfs::close_vfs_node(self); // ...just pray that this doesn't deadlock

        /*
        In all seriousness, the close function acquires locks.
        If you read this while debugging a deadlock a deadlock in the
        scheduler, you might want to check whether you're dropping VfsNodes (maybe through
        open file descriptors) while interrupts are disabled. If so, make sure that you free
        the tasks before you disable interrupts.

        Let's hope that this doesn't happen to you.
         */
    }
}
