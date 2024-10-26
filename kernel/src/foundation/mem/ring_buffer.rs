use crate::foundation::falloc::vec::FVec;
use core::alloc::AllocError;

#[derive(Debug)]
pub struct RingBuffer {
    data: FVec<u8>,
    read_pos: usize,
    write_pos: usize,
}

impl RingBuffer {
    pub fn try_with_size(size: usize) -> Result<Self, AllocError> {
        Ok(Self {
            data: {
                let mut data = FVec::new();
                data.try_resize_with(size, || 0).map_err(|_| AllocError)?;
                data
            },
            read_pos: 0,
            write_pos: 0,
        })
    }

    pub fn read(&mut self, buf: &mut [u8]) -> usize {
        let mut read = 0;
        while read < buf.len() {
            if self.read_pos == self.write_pos {
                break;
            }
            buf[read] = self.data[self.read_pos];
            self.read_pos = (self.read_pos + 1) % self.data.len();
            read += 1;
        }
        read
    }

    pub fn write(&mut self, buf: &[u8]) -> usize {
        let mut written = 0;
        while written < buf.len() {
            if (self.write_pos + 1) % self.data.len() == self.read_pos {
                break;
            }
            self.data[self.write_pos] = buf[written];
            self.write_pos = (self.write_pos + 1) % self.data.len();
            written += 1;
        }
        written
    }
}

#[cfg(feature = "kernel_test")]
mod tests {
    use crate::foundation::mem::ring_buffer::RingBuffer;
    use kernel_test_framework::kernel_test;

    #[kernel_test]
    fn test_ring_buffer() {
        let mut buf = RingBuffer::try_with_size(15).unwrap();
        assert_eq!(buf.write(b"Hello, World!"), 13);
        assert_eq!(buf.write_pos, 13);
        assert_eq!(buf.read_pos, 0);

        let mut read_buf = [0_u8; 5];
        assert_eq!(buf.read(&mut read_buf), 5);
        assert_eq!(&read_buf, b"Hello");
        assert_eq!(buf.read(&mut read_buf), 5);
        assert_eq!(&read_buf, b", Wor");
        assert_eq!(buf.read(&mut read_buf), 3);
        assert_eq!(&read_buf, b"ld!or"); // "ld!" is left in the buffer, "or" is from the previous read
        assert_eq!(buf.read(&mut read_buf), 0);
        assert_eq!(&read_buf, b"ld!or"); // buffer doesn't change when there's no data to read
        assert_eq!(buf.write_pos, 13);
        assert_eq!(buf.read_pos, 13);

        assert_eq!(buf.write(b"Hello, World!"), 13);
        assert_eq!(buf.write_pos, 11);
        assert_eq!(buf.read_pos, 13);

        let mut read_buf = [0_u8; 15];
        assert_eq!(buf.read(&mut read_buf), 13);
        assert_eq!(&read_buf[0..13], b"Hello, World!");
        assert_eq!(&read_buf[13..15], [0, 0]);
    }
}
