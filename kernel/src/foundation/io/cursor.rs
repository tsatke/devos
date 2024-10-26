use crate::foundation::io::{
    Read, ReadError, ReadResult, Seek, SeekError, SeekFrom, Write, WriteError, WriteResult,
};
use core::num::NonZeroUsize;

pub struct Cursor<T> {
    index: usize,
    data: T,
}

impl<T> Cursor<T> {
    pub const fn new(data: T) -> Self {
        Self { index: 0, data }
    }
}

impl<T, E> Read<E> for Cursor<T>
where
    T: AsRef<[E]>,
    E: Copy,
{
    fn read(&mut self, buf: &mut [E]) -> Result<ReadResult, ReadError> {
        let data = self.data.as_ref();

        if self.index == data.len() {
            return Ok(ReadResult::EndOfStream);
        }

        let start = self.index;
        let end = data.len().min(self.index + buf.len());
        let len = end - start;
        buf[..len].copy_from_slice(&data[start..end]);
        self.index = end;
        let len = unsafe {
            debug_assert!(
                len > 0,
                "if len is zero, this should have returned early already"
            );
            NonZeroUsize::new_unchecked(len)
        };
        Ok(ReadResult::Read(len))
    }
}

impl<T, E> Write<E> for Cursor<T>
where
    T: AsMut<[E]>,
    E: Copy,
{
    fn write(&mut self, buf: &[E]) -> Result<WriteResult, WriteError> {
        let data = self.data.as_mut();

        if self.index == data.len() {
            return Ok(WriteResult::NoMoreSpace);
        }

        let start = self.index;
        let end = data.len().min(self.index + buf.len());
        let len = end - start;
        data[start..end].copy_from_slice(&buf[..len]);
        self.index = end;
        let len = unsafe {
            debug_assert!(
                len > 0,
                "if len is zero, this should have returned early already"
            );
            NonZeroUsize::new_unchecked(len)
        };
        Ok(WriteResult::Written(len))
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
