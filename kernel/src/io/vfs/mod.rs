use crate::io::path::{OwnedPath, Path};
use crate::io::vfs::devfs::VirtualDevFs;
use crate::io::vfs::ext2::VirtualExt2Fs;
use alloc::borrow::ToOwned;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use core::sync::atomic::AtomicU64;
use core::sync::atomic::Ordering::Relaxed;
use error::Result;
use error::VfsError;
use spin::RwLock;

pub mod devfs;
mod error;
pub mod ext2;
mod file_system;
mod vfs_node;

pub use file_system::*;
pub use vfs_node::*;

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

pub struct Vfs {
    mounts: RwLock<BTreeMap<OwnedPath, Arc<RwLock<dyn FileSystem>>>>,
}

impl Vfs {
    pub fn mount<P, F>(&self, mount_point: P, fs: F) -> Result<()>
    where
        P: AsRef<Path>,
        F: FileSystem + 'static,
    {
        let fs = Arc::new(RwLock::new(fs));
        self.mounts.write().insert(mount_point.into(), fs);
        Ok(())
    }

    pub fn unmount<P>(&self, mount_point: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let mut guard = self.mounts.write();
        guard.remove::<OwnedPath>(&mount_point.into());
        Ok(())
    }

    pub fn exists<P>(&self, path: P) -> Result<bool>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let (fs, relative_path) = self.find_fs_and_relativize(path)?;
        let result = fs.write().exists(relative_path.as_path())?;
        Ok(result)
    }

    pub fn open<P>(&self, path: P) -> Result<VfsNode>
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
    pub fn close(&self, node: VfsNode) -> Result<()> {
        self.internal_close(&node)
    }

    pub fn read_dir<P>(&self, path: P) -> Result<impl Iterator<Item = DirEntry>>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let (fs, _) = self.find_fs_and_relativize(path)?;
        let vec = fs.write().read_dir(path)?;
        Ok(vec.into_iter())
    }

    pub fn read<B>(&self, node: &VfsNode, mut buf: B, offset: usize) -> Result<usize>
    where
        B: AsMut<[u8]>,
    {
        let buf = buf.as_mut();
        let mut guard = node.fs().write();
        guard.read(node.handle(), buf, offset)
    }

    pub fn write<B>(&self, node: &VfsNode, buf: B, offset: usize) -> Result<usize>
    where
        B: AsRef<[u8]>,
    {
        let buf = buf.as_ref();
        let mut guard = node.fs().write();
        guard.write(node.handle(), buf, offset)
    }

    pub fn truncate(&self, node: &VfsNode, size: usize) -> Result<()> {
        let mut guard = node.fs().write();
        guard.truncate(node.handle(), size)
    }

    pub fn stat(&self, node: &VfsNode) -> Result<Stat> {
        let mut guard = node.fs().write();
        guard.stat(node.handle())
    }

    pub fn stat_path<P>(&self, p: P) -> Result<Stat>
    where
        P: AsRef<Path>,
    {
        let path = p.as_ref();
        let (fs, path) = self.find_fs_and_relativize(path)?;
        let mut guard = fs.write();
        guard.stat_path(path.as_path())
    }

    pub fn create<P>(&self, path: P, ftype: FileType) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let (fs, path) = self.find_fs_and_relativize(path)?;
        let mut guard = fs.write();
        guard.create(path.as_path(), ftype)
    }

    pub fn remove<P>(&self, path: P) -> Result<()>
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
    fn find_fs_and_relativize<P>(&self, path: P) -> Result<(Arc<RwLock<dyn FileSystem>>, OwnedPath)>
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

    fn internal_close(&self, node: &VfsNode) -> Result<()> {
        let mut guard = node.fs().write();
        guard.close(node.handle())
    }
}

/// This method is intended to be called by the VfsNode when it is dropped.
/// It is not intended to be called by you.
fn close_vfs_node(node: &VfsNode) {
    let _ = vfs().internal_close(node);
}
