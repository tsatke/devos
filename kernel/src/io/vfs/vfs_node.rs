use alloc::sync::Arc;
use core::fmt::{Debug, Formatter};
use core::ops::Deref;

use derive_more::Constructor;
use spin::RwLock;
use x86_64::instructions::interrupts;

use crate::io::path::{OwnedPath, Path};
use crate::io::vfs;
use crate::io::vfs::{FileSystem, VfsHandle};

#[derive(Clone)]
pub struct VfsNode {
    inner: Arc<Inner>,
}

impl Deref for VfsNode {
    type Target = Arc<Inner>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(Constructor)]
pub struct Inner {
    /// The path of this node.
    path: OwnedPath,
    /// The file system specific handle.
    handle: VfsHandle,
    fs: Arc<RwLock<dyn FileSystem>>,
}

impl ! Clone for Inner {}

impl Debug for VfsNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("VfsNode")
            .field("path", &self.inner.path)
            .field("handle", &self.inner.handle)
            .finish()
    }
}

impl VfsNode {
    pub(in crate::io::vfs) fn new(
        path: OwnedPath,
        handle: VfsHandle,
        fs: Arc<RwLock<dyn FileSystem>>,
    ) -> Self {
        Self {
            inner: Arc::new(Inner::new(path, handle, fs)),
        }
    }
}

impl Inner {
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

impl Drop for Inner {
    fn drop(&mut self) {
        assert!(
            interrupts::are_enabled(),
            "interrupts must be enabled when dropping a vfsnode"
        ); // best effort, there is no way to guarantee that we don't get preempted right after this, so...
        vfs::close_vfs_node(self); // ...just pray that this doesn't deadlock

        /*
        In all seriousness, the close function acquires locks.
        If you read this while debugging a deadlock in the
        scheduler, you might want to check whether you're dropping VfsNodes (maybe through
        open file descriptors) while interrupts are disabled. If so, make sure that you free
        the threads before you disable interrupts.

        Let's hope that this doesn't happen to you.
         */
    }
}
