use crate::io::{seek_do_restore, Seek, SeekError};
use alloc::boxed::Box;
use core::error::Error;
use core::hint::spin_loop;
use derive_more::Display;

pub trait Write<T> {
    fn write(&mut self, buf: &[T]) -> Result<usize, WriteError>;

    fn write_exact(&mut self, buf: &[T]) -> Result<(), WriteExactError> {
        let mut buf = buf;
        while !buf.is_empty() {
            match self.write(buf) {
                Ok(0) | Err(WriteError::TryAgain) => {
                    spin_loop();
                    continue;
                }
                Ok(n) => buf = &buf[n..],
                Err(WriteError::EndOfStream) => return Err(WriteExactError::IncompleteWrite),
            };
        }
        Ok(())
    }
}

impl<T, U> Write<T> for &'_ mut U
where
    U: Write<T>,
{
    fn write(&mut self, buf: &[T]) -> Result<usize, WriteError> {
        (*self).write(buf)
    }
}

impl<T, U> Write<T> for Box<U>
where
    U: Write<T> + ?Sized,
{
    fn write(&mut self, buf: &[T]) -> Result<usize, WriteError> {
        self.as_mut().write(buf)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Display)]
pub enum WriteError {
    TryAgain,
    EndOfStream,
}

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
    fn write_at(&mut self, buf: &[T], offset: usize) -> Result<usize, WriteAtError>;

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
    fn write_at(&mut self, buf: &[E], offset: usize) -> Result<usize, WriteAtError> {
        seek_do_restore(self, buf, offset, Write::write)
    }

    fn write_at_exact(&mut self, buf: &[E], offset: usize) -> Result<(), WriteAtExactError> {
        seek_do_restore(self, buf, offset, Write::write_exact)
    }
}

pub trait WriteInto<E> {
    fn write_into(&self, out: &mut impl Write<E>) -> Result<(), WriteExactError>;
}
