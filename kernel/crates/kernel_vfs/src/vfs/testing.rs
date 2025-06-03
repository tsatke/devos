use crate::fs::{FileSystem, FsHandle};
use crate::path::{OwnedPath, Path};
use crate::{CloseError, OpenError, ReadError};
use alloc::borrow::ToOwned;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::sync::atomic::AtomicU64;
use core::sync::atomic::Ordering::Relaxed;
use spin::RwLock;

#[derive(Default)]
pub struct TestFs {
    handle_counter: AtomicU64,
    files: BTreeMap<OwnedPath, RwLock<Vec<u8>>>,
    open_files: BTreeMap<FsHandle, OwnedPath>,
}

impl TestFs {
    pub fn insert_file(&mut self, path: impl AsRef<Path>, data: Vec<u8>) {
        let path = path.as_ref().to_owned();
        self.files.insert(path, RwLock::new(data));
    }
}

impl FileSystem for TestFs {
    fn open(&mut self, path: &Path) -> Result<FsHandle, OpenError> {
        let owned = path.to_owned();
        if self.files.contains_key(&owned) {
            let handle = FsHandle::from(self.handle_counter.fetch_add(1, Relaxed));
            self.open_files.insert(handle, owned.clone());
            Ok(handle)
        } else {
            Err(OpenError::NotFound)
        }
    }

    fn close(&mut self, handle: FsHandle) -> Result<(), CloseError> {
        self.open_files
            .remove(&handle)
            .map(|_| ())
            .ok_or(CloseError::NotOpen)
    }

    fn read(
        &mut self,
        handle: FsHandle,
        buf: &mut [u8],
        offset: usize,
    ) -> Result<usize, ReadError> {
        let path = self
            .open_files
            .get(&handle)
            .ok_or(ReadError::InvalidHandle)?;

        // file can't be deleted while it's open, so if we have a handle, it must exist in `self.files`
        let file = self.files.get(path).unwrap();

        let guard = file.read();
        let data = guard.as_slice();
        let file_len = data.len();
        if offset >= file_len {
            return Ok(0);
        }

        let bytes_to_read = file_len - offset;
        let bytes_to_copy = buf.len().min(bytes_to_read);
        buf[..bytes_to_copy].copy_from_slice(&data[offset..offset + bytes_to_copy]);
        Ok(bytes_to_copy)
    }
}

#[cfg(test)]
mod tests {
    use crate::CloseError;
    use crate::fs::FileSystem;
    use crate::path::{OwnedPath, Path};
    use crate::testing::TestFs;

    #[test]
    fn test_open_close() {
        let mut fs = TestFs::default();
        fs.files.insert(OwnedPath::new("/foo"), Default::default());

        assert!(fs.open(Path::new("/bar")).is_err());
        let handle = fs.open(Path::new("/foo")).unwrap();

        assert!(fs.close(handle).is_ok());
        assert_eq!(Err(CloseError::NotOpen), fs.close(handle));
    }
}
