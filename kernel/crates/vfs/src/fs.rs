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
    fn open(&mut self, path: &Path) -> Result<FsHandle, OpenError>;

    fn close(&mut self, handle: FsHandle) -> Result<(), CloseError>;
}
