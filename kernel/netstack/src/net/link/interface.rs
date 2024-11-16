use crate::net::{DataLinkProtocol, Device, MacAddr, ReadFrameResult};
use alloc::boxed::Box;
use alloc::sync::Arc;
use core::future::{poll_fn, Future};
use core::task::Poll;
use derive_more::Constructor;
use foundation::falloc::vec::FVec;
use foundation::future::lock::FutureMutex;
use foundation::future::queue::AsyncBoundedQueue;
use foundation::io::{ReadError, WriteError};

#[derive(Constructor)]
pub struct Frame(DataLinkProtocol, FVec<u8>);

impl Frame {
    pub fn protocol(&self) -> DataLinkProtocol {
        self.0
    }

    pub fn into_data(self) -> FVec<u8> {
        self.1
    }
}

pub struct Interface {
    device: Arc<FutureMutex<Box<dyn Device>>>,
    mac_addr: MacAddr,
    protocol: DataLinkProtocol,

    tx_queue: Arc<AsyncBoundedQueue<Frame>>,
    rx_queue: Arc<AsyncBoundedQueue<Frame>>,
}

impl Interface {
    pub fn new(device: Box<dyn Device>) -> Interface {
        let mac_addr = device.mac_addr();
        let protocol = device.protocol();
        let device = Arc::new(FutureMutex::new(device));
        Self {
            device,
            mac_addr,
            protocol,
            tx_queue: Arc::new(AsyncBoundedQueue::new(64)),
            rx_queue: Arc::new(AsyncBoundedQueue::new(64)),
        }
    }

    pub fn mac_addr(&self) -> MacAddr {
        self.mac_addr
    }

    pub fn protocol(&self) -> DataLinkProtocol {
        self.protocol
    }

    pub async fn send_frame(&self, frame: Frame) -> () {
        self.tx_queue.push(frame).await
    }

    pub fn rx_queue(&self) -> &Arc<AsyncBoundedQueue<Frame>> {
        &self.rx_queue
    }

    pub fn work_rx_queue(&self) -> impl Future<Output=()> + 'static {
        let device = self.device.clone();
        let protocol = self.protocol;
        let rx_queue = self.rx_queue.clone();
        async move {
            loop {
                let mut guard = device.lock().await;
                let mut buf = [0_u8; 2048];
                match guard.read_frame(&mut buf) {
                    Ok(ReadFrameResult::Complete(n)) => {
                        let mut data = FVec::try_with_len(n).unwrap(); // TODO: no panic
                        data.copy_from_slice(&buf[..n]);
                        let frame = Frame(protocol, data);
                        rx_queue.push(frame).await;
                    }
                    Ok(ReadFrameResult::Incomplete(n)) => {
                        todo!("handle incomplete frame");
                    }
                    Err(ReadError::WouldBlock) => {
                        poll_fn(|cx| {
                            guard.wake_when_read_available(cx.waker());
                            Poll::<()>::Pending
                        }).await;
                    }
                    Err(ReadError::EndOfStream) => {
                        return;
                    }
                }
            }
        }
    }

    pub fn work_tx_queue(&self) -> impl Future<Output=()> + 'static {
        let device = self.device.clone();
        let protocol = self.protocol;
        let tx_queue = self.tx_queue.clone();
        async move {
            loop {
                let frame = tx_queue.pop().await;
                debug_assert_eq!(frame.protocol(), protocol);
                let frame = frame.into_data();
                let mut guard = device.lock().await;
                let mut buf = frame.as_ref();
                while !buf.is_empty() {
                    match guard.write_frame(&frame) {
                        Ok(n) => {
                            buf = &buf[n..];
                        }
                        Err(WriteError::WouldBlock) => {
                            poll_fn(|cx| {
                                guard.wake_when_write_available(cx.waker());
                                Poll::<()>::Pending
                            }).await;
                        }
                        Err(WriteError::EndOfStream) => {
                            return;
                        }
                    };
                }
            }
        }
    }
}
