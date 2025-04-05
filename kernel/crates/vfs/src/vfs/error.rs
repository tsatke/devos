use thiserror::Error;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Error)]
pub enum MountError {
    #[error("the mount point is already used by another mount")]
    AlreadyMounted,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Error)]
pub enum UnmountError {
    #[error("not mounted")]
    NotMounted,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Error)]
pub enum ExistsError {}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Error)]
pub enum OpenError {
    #[error("not found")]
    NotFound,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Error)]
pub enum CloseError {
    #[error("not open")]
    NotOpen,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Error)]
pub enum ReadError {
    #[error("filesystem is not open")]
    FileSystemNotOpen,
    #[error("invalid handle")]
    InvalidHandle,
    #[error("end of file")]
    EndOfFile,
}
