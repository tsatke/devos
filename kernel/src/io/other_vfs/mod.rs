use crate::io::other_vfs::devfs::VirtualDevFs;
use crate::io::path::{OwnedPath, Path};
use alloc::borrow::ToOwned;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use core::sync::atomic::AtomicU64;
use spin::RwLock;

pub mod devfs;
mod file_system;

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

    let rootfs_dev = ::ext2::Ext2Fs::try_new(root_drive).expect("root drive must be ext2 for now");

    let devfs = VirtualDevFs::new(FsId::new());
    vfs().mount("/dev", devfs).expect("failed to mount devfs");
}

static FSID_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct FsId(u64);

impl FsId {
    fn new() -> Self {
        Self(FSID_COUNTER.fetch_add(1, ::core::sync::atomic::Ordering::Relaxed))
    }

    fn as_u64(&self) -> u64 {
        self.0
    }
}

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
}

pub struct Vfs {
    file_systems: RwLock<BTreeMap<FsId, Arc<RwLock<dyn FileSystem>>>>,
    mounts: RwLock<BTreeMap<OwnedPath, FsId>>,
}

impl Vfs {
    pub fn mount<P, F>(&self, mount_point: P, fs: F) -> Result<(), VfsError>
    where
        P: AsRef<Path>,
        F: FileSystem + 'static,
    {
        let fsid = fs.fsid();
        let fs = Arc::new(RwLock::new(fs));
        self.file_systems.write().insert(fsid, fs);
        self.mounts.write().insert(mount_point.into(), fsid);
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

    pub fn open<P>(&self, path: P) -> Result<VfsNode, VfsError>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let (fs, relative_path) = self.find_fs_and_relativize(path)?;
        let fsid = fs.read().fsid();
        let handle = fs.write().open(relative_path.as_path())?;
        Ok(VfsNode::new(path.into(), handle, fsid))
    }

    /// This closes a VfsNode. Use this if you need a potential error that might occur
    /// when closing the node. Otherwise, you can just drop the node.
    pub fn close(&self, node: VfsNode) -> Result<(), VfsError> {
        let mut node = node;
        self.internal_close(&mut node)
    }

    pub fn read<B>(&self, node: &VfsNode, mut buf: B, offset: usize) -> Result<usize, VfsError>
    where
        B: AsMut<[u8]>,
    {
        let buf = buf.as_mut();
        let (fs, _) = self.find_fs_and_relativize(node.path())?;
        let mut guard = fs.write();
        guard.read(node.handle(), buf, offset)
    }

    pub fn write<B>(&self, node: &VfsNode, buf: B, offset: usize) -> Result<usize, VfsError>
    where
        B: AsRef<[u8]>,
    {
        let buf = buf.as_ref();
        let (fs, _) = self.find_fs_and_relativize(node.path())?;
        let mut guard = fs.write();
        guard.write(node.handle(), buf, offset)
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
            file_systems: RwLock::new(BTreeMap::new()),
            mounts: RwLock::new(BTreeMap::new()),
        }
    }

    fn find_fs_by_id(&self, fsid: FsId) -> Result<Arc<RwLock<dyn FileSystem>>, VfsError> {
        self.file_systems
            .read()
            .get(&fsid)
            .cloned()
            .ok_or(VfsError::NoSuchFileSystem)
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
        let mut guard = self.mounts.read();
        let original_path = path.as_ref().to_owned().to_string();
        let mut path = path.as_ref().to_owned();
        loop {
            if let Some(fsid) = guard.get::<OwnedPath>(&path) {
                let new_path = original_path.chars().skip(path.len()).collect::<String>();
                return Ok((self.find_fs_by_id(*fsid)?, OwnedPath::from(new_path)));
            }
            if let Some(parent) = path.parent() {
                path = parent.to_owned();
            } else {
                return Err(VfsError::NoSuchFileSystem);
            }
        }
    }

    fn internal_close(&self, node: &mut VfsNode) -> Result<(), VfsError> {
        let path = node.path();
        let (fs, _) = self.find_fs_and_relativize(path)?;
        let mut guard = fs.write();
        guard.close(node.handle())
    }
}

/// This method is intended to be called by the VfsNode when it is dropped.
/// It is not intended to be called by you.
fn close_vfs_node(node: &mut VfsNode) {
    let _ = vfs().internal_close(node);
}
