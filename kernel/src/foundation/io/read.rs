use crate::foundation::io::seek::seek_do_restore;
use crate::foundation::io::seek::{Seek, SeekError};
use core::error::Error;
use core::hint::spin_loop;
use derive_more::Display;

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
                Ok(0) | Err(ReadError::TryAgain) => {
                    spin_loop();
                    continue;
                }
                Ok(n) => buf = &mut buf[n..],
                Err(ReadError::EndOfStream) => return Err(ReadExactError::IncompleteRead),
            };
        }
        Ok(())
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Display)]
pub enum ReadError {
    /// No elements have been read, but more might become
    /// available. Try again.
    TryAgain,
    /// No more elements is or will become available.
    /// The stream has reached the end.
    EndOfStream,
}

impl Error for ReadError {}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Display)]
pub enum ReadExactError {
    Read(ReadError),
    /// The read could not be completed, because not enough
    /// data was available.
    IncompleteRead,
}

impl Error for ReadExactError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ReadExactError::Read(e) => Some(e),
            ReadExactError::IncompleteRead => None,
        }
    }
}

impl From<ReadError> for ReadExactError {
    fn from(value: ReadError) -> Self {
        Self::Read(value)
    }
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

#[derive(Debug, Copy, Clone, Eq, PartialEq, Display)]
pub enum ReadAtError {
    Read(ReadError),
    Seek(SeekError),
}

impl Error for ReadAtError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ReadAtError::Read(e) => Some(e),
            ReadAtError::Seek(e) => Some(e),
        }
    }
}

impl From<ReadError> for ReadAtError {
    fn from(value: ReadError) -> Self {
        Self::Read(value)
    }
}

impl From<SeekError> for ReadAtError {
    fn from(value: SeekError) -> Self {
        Self::Seek(value)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Display)]
pub enum ReadAtExactError {
    ReadExact(ReadExactError),
    Seek(SeekError),
}

impl Error for ReadAtExactError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ReadAtExactError::ReadExact(e) => Some(e),
            ReadAtExactError::Seek(e) => Some(e),
        }
    }
}

impl From<ReadExactError> for ReadAtExactError {
    fn from(value: ReadExactError) -> Self {
        Self::ReadExact(value)
    }
}

impl From<SeekError> for ReadAtExactError {
    fn from(value: SeekError) -> Self {
        Self::Seek(value)
    }
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
