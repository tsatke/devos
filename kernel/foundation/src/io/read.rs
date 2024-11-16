use crate::io::{seek_do_restore, Seek, SeekError};
use alloc::boxed::Box;
use core::hint::spin_loop;
use thiserror::Error;

pub trait Read<T> {
    /// Read from the current position into the provided buffer.
    ///
    /// If a read was successful, the Ok result will indicate how many
    /// bytes have been read - at most `buf.len()`.
    ///
    /// An implementation may perform short reads. To make sure that
    /// the entire buffer is filled with one call, use [`Read::read_exact`].
    fn read(&mut self, buf: &mut [T]) -> Result<usize, ReadError>;

    /// Reads from the current position into the provided buffer, filling the buffer
    /// completely. If successful, `buf.len()` bytes have been read and moved into
    /// `buf`.
    ///
    /// This spins if necessary to wait for more data.
    ///
    /// An error is returned if not enough data is available to fill the whole buffer.
    fn read_exact(&mut self, buf: &mut [T]) -> Result<(), ReadExactError> {
        let mut buf = buf;
        while !buf.is_empty() {
            match self.read(buf) {
                Ok(0) | Err(ReadError::WouldBlock) => {
                    spin_loop();
                    continue;
                }
                Ok(n) => buf = &mut buf[n..],
                Err(ReadError::ResourceExhausted) => return Err(ReadExactError::IncompleteRead),
            };
        }
        Ok(())
    }
}

impl<T, U> Read<T> for &'_ mut U
where
    U: Read<T>,
{
    fn read(&mut self, buf: &mut [T]) -> Result<usize, ReadError> {
        (*self).read(buf)
    }
}

impl<T, U> Read<T> for Box<U>
where
    U: Read<T> + ?Sized,
{
    fn read(&mut self, buf: &mut [T]) -> Result<usize, ReadError> {
        self.as_mut().read(buf)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Error)]
pub enum ReadError {
    /// No elements have been read, but more might become
    /// available. Try again.
    #[error("would block")]
    WouldBlock,
    /// No more elements are or will become available.
    /// The stream has reached the end.
    #[error("resource exhausted")]
    ResourceExhausted,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Error)]
pub enum ReadExactError {
    #[error("read error")]
    Read(#[from] ReadError),
    #[error("a full read could not be completed")]
    IncompleteRead,
}

pub trait ReadAt<T> {
    /// Reads at the given absolute offset.
    ///
    /// See [`Read::read`].
    fn read_at(&mut self, buf: &mut [T], offset: usize) -> Result<usize, ReadAtError>;

    /// Reads ath the given absolute offset and fills the entire buffer.
    ///
    /// See [`Read::read_exact`].
    fn read_at_exact(&mut self, buf: &mut [T], offset: usize) -> Result<(), ReadAtExactError>;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Error)]
pub enum ReadAtError {
    #[error("read error")]
    Read(#[from] ReadError),
    #[error("seek error")]
    Seek(#[from] SeekError),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Error)]
pub enum ReadAtExactError {
    #[error("read exact error")]
    ReadExact(#[from] ReadExactError),
    #[error("seek error")]
    Seek(#[from] SeekError),
}

impl<T, E> ReadAt<E> for T
where
    T: Read<E> + Seek,
{
    fn read_at(&mut self, buf: &mut [E], offset: usize) -> Result<usize, ReadAtError> {
        seek_do_restore(self, buf, offset, Read::read)
    }

    fn read_at_exact(&mut self, buf: &mut [E], offset: usize) -> Result<(), ReadAtExactError> {
        seek_do_restore(self, buf, offset, Read::read_exact)
    }
}
