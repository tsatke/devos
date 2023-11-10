use kernel_api::syscall::Errno;

pub(crate) type Result<T> = core::result::Result<T, VfsError>;

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
    WriteError,
}

impl From<VfsError> for Errno {
    fn from(value: VfsError) -> Self {
        match value {
            VfsError::NoSuchFileSystem => Errno::ENXIO,
            VfsError::NoSuchFile => Errno::ENOENT,
            VfsError::Unsupported => Errno::ENOSYS,
            VfsError::HandleClosed => Errno::EBADF,
            VfsError::ReadError | VfsError::WriteError => Errno::EIO,
        }
    }
}
