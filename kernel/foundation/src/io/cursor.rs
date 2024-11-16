use crate::falloc::vec::FVec;
use crate::io::{Bytes, Read, ReadError, Seek, SeekError, SeekFrom, Write, WriteError};
use alloc::vec::Vec;

pub struct Cursor<T> {
    index: usize,
    data: T,
}

impl<T> Cursor<T> {
    pub const fn new(data: T) -> Self {
        Self { index: 0, data }
    }
}

impl<T: Clone> Clone for Cursor<T> {
    fn clone(&self) -> Self {
        Self {
            index: self.index,
            data: self.data.clone(),
        }
    }
}

impl<T> Cursor<T>
where
    T: AsRef<[u8]>,
{
    pub fn bytes(self) -> Bytes<Self> {
        Bytes::new(self)
    }
}

impl<T, E> Read<E> for Cursor<T>
where
    T: AsRef<[E]>,
    E: Copy,
{
    fn read(&mut self, buf: &mut [E]) -> Result<usize, ReadError> {
        let data = self.data.as_ref();

        if self.index == data.len() {
            return Err(ReadError::ResourceExhausted);
        }

        let start = self.index;
        let end = data.len().min(self.index + buf.len());
        let len = end - start;
        buf[..len].copy_from_slice(&data[start..end]);
        self.index = end;
        Ok(len)
    }
}

impl<T> Write<T> for Cursor<&'_ mut Vec<T>>
where
    T: Copy,
{
    fn write(&mut self, buf: &[T]) -> Result<usize, WriteError> {
        self.data
            .try_reserve(buf.len())
            .map_err(|_| WriteError::ResourceExhausted)?;
        self.data.extend(buf);
        Ok(buf.len())
    }
}

impl<T> Write<T> for Cursor<&'_ mut FVec<T>>
where
    T: Copy,
{
    fn write(&mut self, buf: &[T]) -> Result<usize, WriteError> {
        self.data
            .try_reserve(buf.len())
            .map_err(|_| WriteError::ResourceExhausted)?;
        self.data
            .try_extend(buf.iter().copied())
            .map_err(|_| WriteError::ResourceExhausted)?;
        Ok(buf.len())
    }
}

trait Len {
    fn len(&self) -> usize;
}

impl<T> Len for T
where
    T: AsRef<[u8]>,
{
    fn len(&self) -> usize {
        self.as_ref().len()
    }
}

impl<T> Seek for Cursor<T>
where
    T: Len,
{
    fn seek(&mut self, pos: SeekFrom) -> Result<usize, SeekError> {
        match pos {
            SeekFrom::Start(index) => self.index = index,
            SeekFrom::End(v) => {
                self.index = self
                    .data
                    .len()
                    .checked_add_signed(v)
                    .ok_or(SeekError::SeekOutOfBounds)?;
            }
            SeekFrom::Current(0) => {}
            SeekFrom::Current(v) => {
                self.index = self
                    .index
                    .checked_add_signed(v)
                    .ok_or(SeekError::SeekOutOfBounds)?;
            }
        };
        Ok(self.index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_iter() {
        let data = b"Hello, World!";
        let cursor = Cursor::new(data);
        let bytes = cursor.bytes();

        let mut read = 0;
        for (i, b) in bytes.enumerate() {
            read += 1;
            assert_eq!(b.unwrap(), data[i]);
        }
        assert_eq!(read, data.len());
    }
}
