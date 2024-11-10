use crate::device::Device;
use crate::net::MacAddr;
use alloc::boxed::Box;
use alloc::sync::Arc;
use core::alloc::AllocError;
use core::pin::Pin;
use core::task::{Context, Poll};
use derive_more::{Deref, From};
use foundation::falloc::vec::FVec;
use foundation::io::{Read, ReadError, Write};
use foundation::mem::RingBuffer;
use futures::Stream;
use spin::Mutex;

#[derive(From, Deref)]
pub struct Buffer(FVec<u8>);

pub trait Validator {}

pub struct Interface {
    device: Arc<Mutex<Box<dyn Device>>>,
}

impl Interface {
    pub fn new(device: Box<dyn Device>) -> Interface {
        Self {
            device: Arc::new(Mutex::new(device)),
        }
    }

    pub fn mac_addr(&self) -> MacAddr {
        self.device.lock().mac_addr()
    }

    pub fn frames(&self) -> Result<FrameStream, AllocError> {
        Ok(FrameStream {
            device: self.device.clone(),
            buf: FVec::try_with_len(16384).map_err(|_| AllocError)?.into(),
            index: 0,
        })
    }
}

pub struct FrameStream {
    device: Arc<Mutex<Box<dyn Device>>>,
    buf: RingBuffer<u8>,
    index: usize,
}

impl FrameStream {
    fn next_frame_in_buffer(&mut self) -> Option<Buffer> {
        // FIXME: parse packet depending on the protocol

        let (first, second) = self.buf.current();
        if first.len() == 0 && second.is_none() {
            return None;
        }

        /*
        TODO: parse frame
        1. check whether we have a length available in the data
        2. check whether we have that much data available already
        3. read that much data
        4. parse a frame
        5. discard if invalid
        6. if valid, return
         */

        let len = first.len() + second.map(|b| b.len()).unwrap_or(0);
        let mut data = FVec::try_with_len(len).unwrap();
        self.buf.read_exact(&mut data).unwrap(); // we know that the data is available
        Some(data.into())
    }
}

impl Stream for FrameStream {
    type Item = Buffer;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let Some(next) = self.next_frame_in_buffer() {
            return Poll::Ready(Some(next));
        }

        let device = self.device.clone();
        let mut guard = device.lock();

        let mut buf = [0_u8; 4096];
        let stop_reason = loop {
            let read_count = match guard.read(&mut buf) {
                Ok(n) => n,
                Err(e) => {
                    break e;
                }
            };

            self.buf
                .write(&buf[..read_count])
                .expect("packet longer than rx buffer");
        };

        if let Some(next) = self.next_frame_in_buffer() {
            return Poll::Ready(Some(next));
        }

        match stop_reason {
            ReadError::WouldBlock => {
                guard.wake_upon_data_available(cx.waker());
                Poll::Pending
            }
            ReadError::EndOfStream => Poll::Ready(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::device::Device;
    use crate::executor::{ExecuteResult, Executor};
    use crate::net::{DataLinkProtocol, Interface, MacAddr};
    use alloc::boxed::Box;
    use alloc::sync::Arc;
    use core::sync::atomic::AtomicUsize;
    use core::sync::atomic::Ordering::Relaxed;
    use derive_more::Constructor;
    use foundation::io::{Read, ReadError, Write, WriteError};
    use foundation::mem::RingBuffer;
    use futures::StreamExt;
    use spin::Mutex;

    #[derive(Constructor)]
    struct TestDevice<R, T> {
        rx_buf: Arc<Mutex<R>>,
        tx_buf: Arc<Mutex<T>>,
    }

    impl<R, T> Read<u8> for TestDevice<R, T>
    where
        R: Read<u8>,
    {
        fn read(&mut self, buf: &mut [u8]) -> Result<usize, ReadError> {
            self.rx_buf.lock().read(buf)
        }
    }

    impl<R, T> Write<u8> for TestDevice<R, T>
    where
        T: Write<u8>,
    {
        fn write(&mut self, buf: &[u8]) -> Result<usize, WriteError> {
            self.tx_buf.lock().write(buf)
        }
    }

    impl<R, T> Device for TestDevice<R, T>
    where
        R: Read<u8> + Send + 'static,
        T: Write<u8> + Send + 'static,
    {
        fn mac_addr(&self) -> MacAddr {
            MacAddr::new([0x12, 0x34, 0x56, 0x78, 0x90, 0x19])
        }

        fn protocol(&self) -> DataLinkProtocol {
            DataLinkProtocol::Ethernet
        }
    }

    #[test]
    fn test_interface_frames() {
        let rx = Arc::new(Mutex::new(RingBuffer::try_with_size(16).unwrap()));
        let tx = Arc::new(Mutex::new(RingBuffer::try_with_size(16).unwrap()));
        let bufs = [b"hello", b"world"];

        let executor = Executor::new();

        let device = TestDevice::new(rx.clone(), tx.clone());
        let interface = Interface::new(Box::new(device));
        let count = Arc::new(AtomicUsize::new(0));
        executor
            .spawn({
                let mut frames = interface.frames().unwrap();
                let count = count.clone();
                async move {
                    for (i, &data) in bufs.iter().enumerate() {
                        let frame = frames.next().await.unwrap();
                        assert_eq!(&data, &frame.as_ref(), "frame {i} should match");
                        count.fetch_add(1, Relaxed);
                    }
                }
            })
            .unwrap();

        rx.lock().write_exact(bufs[0]).unwrap();
        assert!(matches!(executor.execute_task(), ExecuteResult::Worked));
        rx.lock().write_exact(bufs[1]).unwrap();
        assert!(matches!(executor.execute_task(), ExecuteResult::Worked));

        assert_eq!(count.load(Relaxed), bufs.len());
    }
}
