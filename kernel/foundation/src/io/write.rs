use crate::io::{seek_do_restore, Seek, SeekError};
use alloc::boxed::Box;
use core::hint::spin_loop;
use thiserror::Error;

pub trait Write<T> {
    fn write(&mut self, buf: &[T]) -> Result<usize, WriteError>;

    fn write_exact(&mut self, buf: &[T]) -> Result<(), WriteExactError> {
        let mut buf = buf;
        while !buf.is_empty() {
            match self.write(buf) {
                Ok(0) | Err(WriteError::WouldBlock) => {
                    spin_loop();
                    continue;
                }
                Ok(n) => buf = &buf[n..],
                Err(WriteError::ResourceExhausted) => return Err(WriteExactError::IncompleteWrite),
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

#[derive(Debug, Copy, Clone, Eq, PartialEq, Error)]
pub enum WriteError {
    #[error("would block")]
    WouldBlock,
    #[error("resource exhausted")]
    ResourceExhausted,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Error)]
pub enum WriteExactError {
    #[error("write error")]
    Write(#[from] WriteError),
    #[error("a full write could not be completed")]
    IncompleteWrite,
}

pub trait WriteAt<T> {
    fn write_at(&mut self, buf: &[T], offset: usize) -> Result<usize, WriteAtError>;

    fn write_at_exact(&mut self, buf: &[T], offset: usize) -> Result<(), WriteAtExactError>;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Error)]
pub enum WriteAtError {
    #[error("write error")]
    Write(#[from] WriteError),
    #[error("seek error")]
    Seek(#[from] SeekError),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Error)]
pub enum WriteAtExactError {
    #[error("write exact error")]
    WriteExact(#[from] WriteExactError),
    #[error("seek error")]
    Seek(#[from] SeekError),
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
