use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use core::sync::atomic::AtomicU64;
use core::sync::atomic::Ordering::Relaxed;

use kernel_vfs::fs::{FileSystem, FsHandle};
use kernel_vfs::path::Path;
use kernel_vfs::{CloseError, FsError, OpenError, ReadError, Stat, StatError, WriteError};

use crate::serial_print;

pub struct DevFs {
    handles: BTreeMap<FsHandle, Box<dyn DevFile + 'static>>,
}

trait DevFile: Send + Sync {
    fn read(&self, buf: &mut [u8], offset: usize) -> Result<usize, ReadError>;
    fn write(&self, buf: &[u8], offset: usize) -> Result<usize, WriteError>;
}

struct DevNull;
impl DevFile for DevNull {
    fn read(&self, buf: &mut [u8], _: usize) -> Result<usize, ReadError> {
        buf.fill(0);
        Ok(buf.len())
    }

    fn write(&self, buf: &[u8], _: usize) -> Result<usize, WriteError> {
        Ok(buf.len())
    }
}

struct Serial;

impl DevFile for Serial {
    fn read(&self, _: &mut [u8], _: usize) -> Result<usize, ReadError> {
        Err(ReadError::EndOfFile)
    }

    fn write(&self, buf: &[u8], _: usize) -> Result<usize, WriteError> {
        let v = core::str::from_utf8(buf).map_err(|_| WriteError::WriteFailed)?;
        serial_print!("{v}");
        Ok(buf.len())
    }
}

impl Default for DevFs {
    fn default() -> Self {
        Self::new()
    }
}

impl DevFs {
    pub fn new() -> Self {
        DevFs {
            handles: BTreeMap::new(),
        }
    }
}

impl FileSystem for DevFs {
    fn open(&mut self, path: &Path) -> Result<FsHandle, OpenError> {
        static FS_COUNTER: AtomicU64 = AtomicU64::new(0);

        let file: Box<dyn DevFile> = match path.as_ref() {
            "/null" => Box::new(DevNull),
            "/serial" => Box::new(Serial),
            _ => return Err(OpenError::NotFound),
        };
        let handle = FsHandle::from(FS_COUNTER.fetch_add(1, Relaxed));
        self.handles.insert(handle, file);
        Ok(handle)
    }

    fn close(&mut self, handle: FsHandle) -> Result<(), CloseError> {
        if self.handles.remove(&handle).is_none() {
            Err(CloseError::NotOpen)
        } else {
            Ok(())
        }
    }

    fn read(
        &mut self,
        handle: FsHandle,
        buf: &mut [u8],
        offset: usize,
    ) -> Result<usize, ReadError> {
        let file = &self.handles.get(&handle).ok_or(FsError::InvalidHandle)?;
        file.read(buf, offset)
    }

    fn write(&mut self, handle: FsHandle, buf: &[u8], offset: usize) -> Result<usize, WriteError> {
        let file = &self.handles.get(&handle).ok_or(FsError::InvalidHandle)?;
        file.write(buf, offset)
    }

    fn stat(&mut self, _handle: FsHandle, _stat: &mut Stat) -> Result<(), StatError> {
        todo!()
    }
}
