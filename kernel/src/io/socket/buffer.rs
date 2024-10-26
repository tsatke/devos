use crate::foundation::mem::RingBuffer;
use core::alloc::AllocError;
use spin::Mutex;

/// A ring buffer for socket communication.
pub struct SocketBuffer {
    inner: Mutex<RingBuffer>,
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

    pub fn read(&self, buf: &mut [u8]) -> usize {
        self.inner.lock().read(buf)
    }

    pub fn write(&self, buf: &[u8]) -> usize {
        self.inner.lock().write(buf)
    }
}
