use crate::io::path::{OwnedPath, Path};
use crate::io::vfs::devfs::VirtualDevFs;
use alloc::borrow::ToOwned;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use core::sync::atomic::AtomicU64;
use core::sync::atomic::Ordering::Relaxed;
use spin::RwLock;

pub mod devfs;
pub mod ext2;
mod file_system;

use crate::io::vfs::ext2::VirtualExt2Fs;
pub use file_system::*;

static VFS: Vfs = Vfs::new();

pub fn vfs() -> &'static Vfs {
    &VFS
}

pub fn init() {
    let root_drive = ide::drives()
        .nth(1)
        .expect("we need at least one additional IDE drive for now")
        .clone();

    let ext2fs = VirtualExt2Fs::new(
        FsId::new(),
        ::ext2::Ext2Fs::try_new(root_drive).expect("root drive must be ext2 for now"),
    );
    vfs().mount("/", ext2fs).expect("failed to mount root fs");

    let devfs = VirtualDevFs::new(FsId::new());
    vfs().mount("/dev", devfs).expect("failed to mount devfs");
}

static FSID_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct FsId(u64);

impl FsId {
    fn new() -> Self {
        Self(FSID_COUNTER.fetch_add(1, Relaxed))
    }
}

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
}

impl Drop for VfsNode {
    fn drop(&mut self) {
        close_vfs_node(self); // just pray that this doesn't deadlock

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

#[derive(Debug)]
pub enum VfsError {
    /// There is no file system associated with the given path or file system id.
    NoSuchFileSystem,
    /// The given path does not exist.
    NoSuchFile,
    /// The operation is not supported by this file system.
    Unsupported,
    /// The handle is either closed or invalid. This should not happen and is a bug.
    /// This can be returned by file system implementations to cover a case where
    /// the handle is closed or not opened yet, but it is received as an argument to
    /// a function from the vfs.
    HandleClosed,
    ReadError,
}

pub struct Vfs {
    mounts: RwLock<BTreeMap<OwnedPath, Arc<RwLock<dyn FileSystem>>>>,
}

impl Vfs {
    pub fn mount<P, F>(&self, mount_point: P, fs: F) -> Result<(), VfsError>
    where
        P: AsRef<Path>,
        F: FileSystem + 'static,
    {
        let fs = Arc::new(RwLock::new(fs));
        self.mounts.write().insert(mount_point.into(), fs);
        Ok(())
    }

    pub fn unmount<P>(&self, mount_point: P) -> Result<(), VfsError>
    where
        P: AsRef<Path>,
    {
        let mut guard = self.mounts.write();
        guard.remove::<OwnedPath>(&mount_point.into());
        Ok(())
    }

    pub fn exists<P>(&self, path: P) -> Result<bool, VfsError>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let (fs, relative_path) = self.find_fs_and_relativize(path)?;
        let result = fs.write().exists(relative_path.as_path())?;
        Ok(result)
    }

    pub fn open<P>(&self, path: P) -> Result<VfsNode, VfsError>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let (fs, relative_path) = self.find_fs_and_relativize(path)?;
        let handle = fs.write().open(relative_path.as_path())?;
        Ok(VfsNode::new(path.into(), handle, fs))
    }

    /// This closes a VfsNode. Use this if you need a potential error that might occur
    /// when closing the node. Otherwise, you can just drop the node.
    pub fn close(&self, node: VfsNode) -> Result<(), VfsError> {
        self.internal_close(&node)
    }

    pub fn read<B>(&self, node: &VfsNode, mut buf: B, offset: usize) -> Result<usize, VfsError>
    where
        B: AsMut<[u8]>,
    {
        let buf = buf.as_mut();
        let mut guard = node.fs.write();
        guard.read(node.handle(), buf, offset)
    }

    pub fn write<B>(&self, node: &VfsNode, buf: B, offset: usize) -> Result<usize, VfsError>
    where
        B: AsRef<[u8]>,
    {
        let buf = buf.as_ref();
        let mut guard = node.fs.write();
        guard.write(node.handle(), buf, offset)
    }

    pub fn truncate(&self, node: &VfsNode, size: usize) -> Result<(), VfsError> {
        let mut guard = node.fs.write();
        guard.truncate(node.handle(), size)
    }

    pub fn stat(&self, node: &VfsNode) -> Result<Stat, VfsError> {
        let guard = node.fs.read();
        guard.stat(node.handle())
    }

    pub fn create<P>(&self, path: P, ftype: FileType) -> Result<(), VfsError>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let (fs, path) = self.find_fs_and_relativize(path)?;
        let mut guard = fs.write();
        guard.create(path.as_path(), ftype)
    }

    pub fn remove<P>(&self, path: P) -> Result<(), VfsError>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let (fs, path) = self.find_fs_and_relativize(path)?;
        let mut guard = fs.write();
        guard.remove(path.as_path())
    }
}

impl Vfs {
    const fn new() -> Self {
        Self {
            mounts: RwLock::new(BTreeMap::new()),
        }
    }

    /// Finds the appropriate file system for the given path and relativizes the path
    /// relative to the mount point of the found fs.
    /// The returned path can be passed into the file system's methods.
    fn find_fs_and_relativize<P>(
        &self,
        path: P,
    ) -> Result<(Arc<RwLock<dyn FileSystem>>, OwnedPath), VfsError>
    where
        P: AsRef<Path>,
    {
        let guard = self.mounts.read();
        let original_path = path.as_ref().to_owned().to_string();
        let mut path = path.as_ref().to_owned();
        loop {
            if let Some(fs) = guard.get::<OwnedPath>(&path) {
                let new_path = original_path.chars().skip(path.len()).collect::<String>();
                return Ok((fs.clone(), OwnedPath::from(new_path)));
            }
            if let Some(parent) = path.parent() {
                path = parent.to_owned();
            } else {
                return Err(VfsError::NoSuchFileSystem);
            }
        }
    }

    fn internal_close(&self, node: &VfsNode) -> Result<(), VfsError> {
        let mut guard = node.fs.write();
        guard.close(node.handle())
    }
}

/// This method is intended to be called by the VfsNode when it is dropped.
/// It is not intended to be called by you.
fn close_vfs_node(node: &VfsNode) {
    let _ = vfs().internal_close(node);
}
