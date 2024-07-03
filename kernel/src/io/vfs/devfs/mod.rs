use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::sync::atomic::AtomicU64;
use core::sync::atomic::Ordering::Relaxed;

use x86_64::structures::paging::PhysFrame;

use kernel_api::syscall::Stat;

use crate::io::path::Path;
use crate::io::vfs::{DirEntry, FileSystem, FileType, FsId, VfsHandle};
use crate::io::vfs::devfs::zero::Zero;
use crate::io::vfs::error::{Result, VfsError};

mod fb;
mod stdio;
mod zero;

static HANDLE_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Helper to create a new handle.
fn next_handle() -> VfsHandle {
    VfsHandle::new(HANDLE_COUNTER.fetch_add(1, Relaxed))
}

pub trait DevFile: Send + Sync {
    fn read(&self, buf: &mut [u8], offset: usize) -> Result<usize>;

    fn write(&mut self, buf: &[u8], offset: usize) -> Result<usize>;

    fn stat(&self, stat: &mut Stat) -> Result<()>;

    fn physical_memory(&self) -> Result<Option<Box<dyn Iterator<Item=PhysFrame> + '_>>> {
        Ok(None)
    }
}

pub type OpenFileFn<'a> = dyn Fn() -> Box<dyn DevFile> + 'a + Send + Sync;

pub struct VirtualDevFs<'a> {
    fsid: FsId,
    handles: BTreeMap<VfsHandle, Box<dyn DevFile>>,
    open_functions: BTreeMap<String, Box<OpenFileFn<'a>>>,
}

impl<'a> VirtualDevFs<'a> {
    pub fn new(fsid: FsId) -> Self {
        let mut res = Self {
            fsid,
            handles: BTreeMap::new(),
            open_functions: BTreeMap::new(),
        };

        res.register_file("/zero", || Box::new(Zero));
        res.register_file("/stdin", || Box::new(stdio::STDIN));
        res.register_file("/stdout", || Box::new(stdio::STDOUT));
        res.register_file("/stderr", || Box::new(stdio::STDERR));

        for (i, fb) in fb::find_fbs().enumerate() {
            res.register_file(format!("/fb{i}"), move || Box::new(fb.clone()));
        }

        res
    }

    pub fn register_file<F: Fn() -> Box<dyn DevFile> + 'a + Send + Sync>(&mut self, path: impl AsRef<str>, open_fn: F) {
        self.open_functions.insert(path.as_ref().to_string(), Box::new(open_fn));
    }
}

impl VirtualDevFs<'_> {
    fn get_impl(&self, handle: VfsHandle) -> Result<&dyn DevFile> {
        match self.handles.get(&handle) {
            Some(v) => Ok(v.as_ref()),
            None => Err(VfsError::NoSuchFile),
        }
    }

    fn get_impl_mut(&mut self, handle: VfsHandle) -> Result<&mut dyn DevFile> {
        match self.handles.get_mut(&handle) {
            Some(v) => Ok(v.as_mut()),
            None => Err(VfsError::NoSuchFile),
        }
    }
}

impl FileSystem for VirtualDevFs<'_> {
    fn fsid(&self) -> FsId {
        self.fsid
    }

    fn open(&mut self, path: &Path) -> Result<VfsHandle> {
        let implementation = self.open_functions
            .get(path.to_string().as_str())
            .ok_or(VfsError::NoSuchFile)?
            ();
        let handle = next_handle();
        self.handles.insert(handle, implementation);
        Ok(handle)
    }

    fn close(&mut self, handle: VfsHandle) -> Result<()> {
        self.handles.remove(&handle).ok_or(VfsError::HandleClosed)?;
        Ok(())
    }

    fn read_dir(&mut self, _path: &Path) -> Result<Vec<DirEntry>> {
        todo!("read_dir not yet implemented for VirtualDevFs")
    }

    fn read(&mut self, handle: VfsHandle, buf: &mut [u8], offset: usize) -> Result<usize> {
        self.get_impl(handle)?.read(buf, offset)
    }

    fn write(&mut self, handle: VfsHandle, buf: &[u8], offset: usize) -> Result<usize> {
        self.get_impl_mut(handle)?.write(buf, offset)
    }

    fn truncate(&mut self, _handle: VfsHandle, _size: usize) -> Result<()> {
        todo!()
    }

    fn stat(&mut self, handle: VfsHandle, stat: &mut Stat) -> Result<()> {
        self.get_impl(handle)?.stat(stat)
    }

    fn create(&mut self, _: &Path, _: FileType) -> Result<()> {
        Err(VfsError::Unsupported)
    }

    fn remove(&mut self, _: &Path) -> Result<()> {
        Err(VfsError::Unsupported)
    }

    fn physical_memory(&self, handle: VfsHandle) -> Result<Option<Box<dyn Iterator<Item=PhysFrame> + '_>>> {
        self.get_impl(handle)?.physical_memory()
    }
}
