use crate::io::path::{OwnedPath, Path};
use crate::io::vfs::devfs::zero::Zero;
use crate::io::vfs::{FileSystem, FileType, FsId, Stat, VfsError, VfsHandle};
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::sync::atomic::AtomicU64;
use core::sync::atomic::Ordering::Relaxed;

mod zero;

static HANDLE_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Helper to create a new handle.
fn next_handle() -> VfsHandle {
    VfsHandle::new(HANDLE_COUNTER.fetch_add(1, Relaxed))
}

pub trait DevFile: Send + Sync {
    fn read(&self, buf: &mut [u8], offset: usize) -> Result<usize, VfsError>;
    fn write(&mut self, buf: &[u8], offset: usize) -> Result<usize, VfsError>;
}

pub struct VirtualDevFs {
    fsid: FsId,
    handles: BTreeMap<VfsHandle, Box<dyn DevFile>>,
}

impl VirtualDevFs {
    pub fn new(fsid: FsId) -> Self {
        Self {
            fsid,
            handles: BTreeMap::new(),
        }
    }
}

impl VirtualDevFs {
    fn get_impl(&self, handle: VfsHandle) -> Result<&dyn DevFile, VfsError> {
        match self.handles.get(&handle) {
            Some(v) => Ok(v.as_ref()),
            None => Err(VfsError::NoSuchFile),
        }
    }

    fn get_impl_mut(&mut self, handle: VfsHandle) -> Result<&mut dyn DevFile, VfsError> {
        match self.handles.get_mut(&handle) {
            Some(v) => Ok(v.as_mut()),
            None => Err(VfsError::NoSuchFile),
        }
    }
}

impl FileSystem for VirtualDevFs {
    fn fsid(&self) -> FsId {
        self.fsid
    }

    fn open(&mut self, path: &Path) -> Result<VfsHandle, VfsError> {
        let implementation: Box<dyn DevFile> = match path.as_str() {
            "/zero" => Box::new(Zero),
            _ => return Err(VfsError::NoSuchFile),
        };
        let handle = next_handle();
        self.handles.insert(handle, implementation);
        Ok(handle)
    }

    fn close(&mut self, handle: VfsHandle) -> Result<(), VfsError> {
        self.handles.remove(&handle).ok_or(VfsError::HandleClosed)?;
        Ok(())
    }

    fn read_dir(&mut self, _path: &Path) -> Result<Vec<OwnedPath>, VfsError> {
        todo!("read_dir not yet implemented for VirtualDevFs")
    }

    fn read(
        &mut self,
        handle: VfsHandle,
        buf: &mut [u8],
        offset: usize,
    ) -> Result<usize, VfsError> {
        self.get_impl(handle)?.read(buf, offset)
    }

    fn write(&mut self, handle: VfsHandle, buf: &[u8], offset: usize) -> Result<usize, VfsError> {
        self.get_impl_mut(handle)?.write(buf, offset)
    }

    fn truncate(&mut self, _handle: VfsHandle, _size: usize) -> Result<(), VfsError> {
        todo!()
    }

    fn stat(&self, _handle: VfsHandle) -> Result<Stat, VfsError> {
        todo!()
    }

    fn create(&mut self, _: &Path, _: FileType) -> Result<(), VfsError> {
        Err(VfsError::Unsupported)
    }

    fn remove(&mut self, _: &Path) -> Result<(), VfsError> {
        Err(VfsError::Unsupported)
    }
}
