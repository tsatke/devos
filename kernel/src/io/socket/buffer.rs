use crate::foundation::io::{Read, ReadError, ReadResult, Write, WriteError, WriteResult};
use crate::foundation::mem::RingBuffer;
use core::alloc::AllocError;
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
    fn read(&mut self, buf: &mut [u8]) -> Result<ReadResult, ReadError> {
        self.inner.lock().read(buf)
    }
}

impl Write<u8> for SocketBuffer {
    fn write(&mut self, buf: &[u8]) -> Result<WriteResult, WriteError> {
        self.inner.lock().write(buf)
    }
}
