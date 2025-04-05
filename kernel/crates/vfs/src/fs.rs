use crate::path::Path;
use crate::{CloseError, OpenError};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct FsHandle(u64);

impl From<u64> for FsHandle {
    fn from(handle: u64) -> Self {
        FsHandle(handle)
    }
}

pub trait FileSystem {
    /// # Errors
    /// Returns an error if the path does not point to a file, or if there
    /// was an underlying error during opening (such as a hardware error).
    fn open(&mut self, path: &Path) -> Result<FsHandle, OpenError>;

    /// # Errors
    /// Returns an error if the handle is invalid or already closed,
    /// or if there was an underlying error during closing (such as
    /// a hardware error).
    fn close(&mut self, handle: FsHandle) -> Result<(), CloseError>;
}
