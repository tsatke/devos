use crate::falloc::vec::FVec;
use crate::io::{Read, ReadError, Write, WriteError};
use core::alloc::AllocError;

#[derive(Debug)]
pub struct RingBuffer<T> {
    data: FVec<T>,
    read_pos: usize,
    write_pos: usize,
}

impl<T> From<FVec<T>> for RingBuffer<T> {
    fn from(value: FVec<T>) -> Self {
        Self {
            data: value,
            read_pos: 0,
            write_pos: 0,
        }
    }
}

impl<T: Default> RingBuffer<T> {
    pub fn try_with_size(size: usize) -> Result<Self, AllocError> {
        Self::try_with_size_with(size, T::default)
    }
}

impl<T> RingBuffer<T> {
    pub fn try_with_size_with<F>(size: usize, f: F) -> Result<Self, AllocError>
    where
        F: FnMut() -> T,
    {
        Ok(Self {
            data: {
                let mut data = FVec::new();
                data.try_resize_with(size, f).map_err(|_| AllocError)?;
                data
            },
            read_pos: 0,
            write_pos: 0,
        })
    }

    pub fn current(&self) -> (&[T], Option<&[T]>) {
        if self.read_pos <= self.write_pos {
            (&self.data[self.read_pos..self.write_pos], None)
        } else {
            (
                &self.data[self.read_pos..],
                Some(&self.data[..self.write_pos]),
            )
        }
    }
}

impl<T: Copy> Read<T> for RingBuffer<T> {
    fn read(&mut self, buf: &mut [T]) -> Result<usize, ReadError> {
        if buf.is_empty() {
            return Ok(0);
        }

        let mut read = 0;
        while read < buf.len() {
            if self.read_pos == self.write_pos {
                break;
            }
            buf[read] = self.data[self.read_pos];
            self.read_pos = (self.read_pos + 1) % self.data.len();
            read += 1;
        }
        if read == 0 {
            Err(ReadError::WouldBlock)
        } else {
            Ok(read)
        }
    }
}

impl<T: Copy> Write<T> for RingBuffer<T> {
    fn write(&mut self, buf: &[T]) -> Result<usize, WriteError> {
        if buf.is_empty() {
            return Ok(0);
        }

        let mut written = 0;
        while written < buf.len() {
            if (self.write_pos + 1) % self.data.len() == self.read_pos {
                break;
            }
            self.data[self.write_pos] = buf[written];
            self.write_pos = (self.write_pos + 1) % self.data.len();
            written += 1;
        }
        if written == 0 {
            Err(WriteError::WouldBlock)
        } else {
            Ok(written)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::io::ReadError;
    use crate::mem::RingBuffer;

    #[test]
    fn test_ring_buffer() {
        let mut buf = RingBuffer::try_with_size(15).unwrap();
        assert_eq!(buf.write(b"Hello, World!").unwrap(), 13);
        assert_eq!(buf.write_pos, 13);
        assert_eq!(buf.read_pos, 0);

        let mut read_buf = [0_u8; 5];
        assert_eq!(buf.read(&mut read_buf).unwrap(), 5);
        assert_eq!(&read_buf, b"Hello");

        assert_eq!(buf.read(&mut read_buf).unwrap(), 5);
        assert_eq!(&read_buf, b", Wor");

        assert_eq!(buf.read(&mut read_buf).unwrap(), 3);
        assert_eq!(&read_buf, b"ld!or"); // "ld!" is left in the buffer, "or" is from the previous read

        assert_eq!(buf.read(&mut read_buf), Err(ReadError::WouldBlock));
        assert_eq!(&read_buf, b"ld!or"); // buffer doesn't change when there's no data to read
        assert_eq!(buf.write_pos, 13);
        assert_eq!(buf.read_pos, 13);

        assert_eq!(buf.write(b"Hello, World!").unwrap(), 13);
        assert_eq!(buf.write_pos, 11);
        assert_eq!(buf.read_pos, 13);

        let mut read_buf = [0_u8; 15];
        assert_eq!(buf.read(&mut read_buf).unwrap(), 13);
        assert_eq!(&read_buf[0..13], b"Hello, World!");
        assert_eq!(&read_buf[13..15], [0, 0]);
    }

    #[test]
    fn test_current() {
        let mut buf = RingBuffer::try_with_size(12).unwrap();
        buf.write_exact(b"hello").unwrap();
        buf.read_exact(&mut [0; 5]).unwrap();
        buf.write_exact(b"hellohello").unwrap();
        let first: &[u8] = b"hellohe";
        let second: &[u8] = b"llo";
        assert_eq!((first, Some(second)), buf.current());
    }
}
