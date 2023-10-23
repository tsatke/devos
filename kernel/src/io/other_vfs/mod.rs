use crate::io::path::{OwnedPath, Path};
use alloc::borrow::ToOwned;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use core::sync::atomic::AtomicU64;
use node::VfsNode;
use spin::RwLock;

mod file_system;
mod node;

pub use file_system::*;
pub use node::*;

pub mod devfs;
pub mod ext2;

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
}

pub enum VfsError {
    NoSuchFileSystem,
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
        let (fs, path) = self.find_fs_and_relativize(path)?;
        let fsid = fs.read().fsid();
        let handle = fs.write().open(path.as_path())?;
        Ok(VfsNode::new(path.into(), handle, fsid))
    }

    pub fn close(&self, node: VfsNode) -> Result<(), VfsError> {
        let path = node.path();
        let (fs, _) = self.find_fs_and_relativize(path)?;
        let mut guard = fs.write();
        guard.close(node.handle())
    }

    pub fn read<B>(&self, node: &VfsNode, mut buf: B, offset: usize) -> Result<usize, VfsError>
    where
        B: AsMut<[u8]>,
    {
        let buf = buf.as_mut();
        let (fs, path) = self.find_fs_and_relativize(node.path())?;
        let mut guard = fs.write();
        guard.read(node.handle(), buf, offset)
    }

    pub fn write<B>(&self, node: &VfsNode, buf: B, offset: usize) -> Result<usize, VfsError>
    where
        B: AsRef<[u8]>,
    {
        let buf = buf.as_ref();
        let (fs, path) = self.find_fs_and_relativize(node.path())?;
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
}
