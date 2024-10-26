use crate::foundation::io::seek::seek_do_restore;
use crate::foundation::io::{Seek, SeekError};
use core::error::Error;
use core::hint::spin_loop;
use core::num::NonZeroUsize;
use derive_more::Display;

pub trait Write<T> {
    fn write(&mut self, buf: &[T]) -> Result<WriteResult, WriteError>;

    fn write_exact(&mut self, buf: &[T]) -> Result<(), WriteExactError> {
        let mut buf = buf;
        while !buf.is_empty() {
            match self.write(buf) {
                Ok(WriteResult::Written(n)) => buf = &buf[n.get()..],
                Ok(WriteResult::TryAgain) => {
                    spin_loop();
                    continue;
                }
                Ok(WriteResult::NoMoreSpace) => return Err(WriteExactError::IncompleteWrite),
                Err(e) => return Err(WriteExactError::Write(e)),
            };
        }
        Ok(())
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Display)]
pub enum WriteResult {
    /// The indicated amount of elements has been written.
    Written(NonZeroUsize),
    /// Currently, no elements can be written, but the operation
    /// may be successful later. Try again.
    TryAgain,
    /// There is no more space that could accommodate more elements.
    NoMoreSpace,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Display)]
pub enum WriteError {}

impl Error for WriteError {}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Display)]
pub enum WriteExactError {
    Write(WriteError),
    IncompleteWrite,
}

impl Error for WriteExactError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            WriteExactError::Write(e) => Some(e),
            WriteExactError::IncompleteWrite => None,
        }
    }
}

impl From<WriteError> for WriteExactError {
    fn from(value: WriteError) -> Self {
        Self::Write(value)
    }
}

pub trait WriteAt<T> {
    fn write_at(&mut self, buf: &[T], offset: usize) -> Result<WriteResult, WriteAtError>;

    fn write_at_exact(&mut self, buf: &[T], offset: usize) -> Result<(), WriteAtExactError>;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Display)]
pub enum WriteAtError {
    Write(WriteError),
    Seek(SeekError),
}

impl Error for WriteAtError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            WriteAtError::Write(e) => Some(e),
            WriteAtError::Seek(e) => Some(e),
        }
    }
}

impl From<WriteError> for WriteAtError {
    fn from(err: WriteError) -> Self {
        WriteAtError::Write(err)
    }
}

impl From<SeekError> for WriteAtError {
    fn from(err: SeekError) -> Self {
        WriteAtError::Seek(err)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Display)]
pub enum WriteAtExactError {
    WriteExact(WriteExactError),
    Seek(SeekError),
}

impl Error for WriteAtExactError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            WriteAtExactError::WriteExact(e) => Some(e),
            WriteAtExactError::Seek(e) => Some(e),
        }
    }
}

impl From<WriteExactError> for WriteAtExactError {
    fn from(value: WriteExactError) -> Self {
        Self::WriteExact(value)
    }
}

impl From<SeekError> for WriteAtExactError {
    fn from(value: SeekError) -> Self {
        Self::Seek(value)
    }
}

impl<T, E> WriteAt<E> for T
where
    T: Write<E> + Seek,
{
    fn write_at(&mut self, buf: &[E], offset: usize) -> Result<WriteResult, WriteAtError> {
        seek_do_restore(self, buf, offset, Write::write)
    }

    fn write_at_exact(&mut self, buf: &[E], offset: usize) -> Result<(), WriteAtExactError> {
        seek_do_restore(self, buf, offset, Write::write_exact)
    }
}
