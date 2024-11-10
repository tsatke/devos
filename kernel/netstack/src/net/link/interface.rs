use crate::net::{DataLinkProtocol, Device, MacAddr, ReadFrameResult};
use alloc::boxed::Box;
use alloc::sync::Arc;
use core::alloc::AllocError;
use core::pin::Pin;
use core::task::{Context, Poll};
use derive_more::Constructor;
use foundation::falloc::vec::FVec;
use foundation::future::lock::FutureMutex;
use foundation::io::ReadError;
use futures::Stream;

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
        }
    }

    pub fn mac_addr(&self) -> MacAddr {
        self.mac_addr
    }

    pub fn protocol(&self) -> DataLinkProtocol {
        self.protocol
    }

    pub fn frames(&self) -> Result<FrameStream, AllocError> {
        Ok(FrameStream {
            device: self.device.clone(),
            protocol: self.protocol,
        })
    }
}

pub struct FrameStream {
    device: Arc<FutureMutex<Box<dyn Device>>>,
    protocol: DataLinkProtocol,
}

impl Stream for FrameStream {
    type Item = Frame;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // TODO: does this size make sense?
        let mut frame = match FVec::try_with_capacity(2048) {
            Ok(v) => v,
            Err(_) => {
                cx.waker().wake_by_ref();
                return Poll::Pending;
            }
        };

        let Some(mut guard) = self.device.try_lock() else {
            cx.waker().wake_by_ref();
            return Poll::Pending;
        };

        let mut buf = [0_u8; 2048];
        loop {
            match guard.read_frame(&mut buf) {
                Ok(ReadFrameResult::Incomplete(n)) => {
                    // TODO: log packet loss if this errors
                    let _ = frame.try_extend(buf.iter().copied().take(n));
                }
                Ok(ReadFrameResult::Complete(n)) => {
                    // TODO: log packet loss if this errors
                    let _ = frame.try_extend(buf.iter().copied().take(n));
                    return Poll::Ready(Some(Frame::new(self.protocol, frame)));
                }
                Err(ReadError::WouldBlock) => {
                    guard.wake_upon_data_available(cx.waker());
                    return Poll::Pending;
                }
                Err(ReadError::EndOfStream) => {
                    return Poll::Ready(None);
                }
            };
        }
    }
}
