#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum IoError {
    NotImplemented,
    Unsupported,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ReadError {
    IoError(IoError),
    InvalidOffset(usize),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum WriteError {
    IoError(IoError),
    InvalidOffset(usize),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum OpenError {
    IoError(IoError),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum LookupError {
    IoError(IoError),
    ReadError(ReadError),
    NoSuchEntry,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum CreateError {
    IoError(IoError),
    MountError(MountError),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum MountError {
    IoError(IoError),
    ReadError(ReadError),
    LookupError(LookupError),
    ExistsButShouldNot,
    NotDirectory,
}

macro_rules! from_error_impl {
    (from $from:ident for $name:ident) => {
        impl From<$from> for $name {
            fn from(value: $from) -> Self {
                Self::$from(value)
            }
        }
    };
}

from_error_impl!(from IoError for ReadError);
from_error_impl!(from IoError for WriteError);
from_error_impl!(from IoError for OpenError);
from_error_impl!(from IoError for LookupError);
from_error_impl!(from IoError for CreateError);
from_error_impl!(from IoError for MountError);
from_error_impl!(from LookupError for MountError);
from_error_impl!(from MountError for CreateError);
from_error_impl!(from ReadError for LookupError);
from_error_impl!(from ReadError for MountError);
