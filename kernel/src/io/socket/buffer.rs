use core::alloc::AllocError;
use foundation::io::{Read, ReadError, Write, WriteError};
use foundation::mem::RingBuffer;
use spin::Mutex;

/// A ring buffer for socket communication.
pub struct SocketBuffer {
    inner: Mutex<RingBuffer<u8>>,
}

impl SocketBuffer {
    pub fn try_new() -> Result<Self, AllocError> {
        Ok(Self::try_with_size(256 * 1024)?)
    }

    fn try_with_size(size: usize) -> Result<Self, AllocError> {
        Ok(Self {
            inner: Mutex::new(RingBuffer::try_with_size(size)?),
        })
    }
}

impl Read<u8> for SocketBuffer {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ReadError> {
        self.inner.lock().read(buf)
    }
}

impl Write<u8> for SocketBuffer {
    fn write(&mut self, buf: &[u8]) -> Result<usize, WriteError> {
        self.inner.lock().write(buf)
    }
}
