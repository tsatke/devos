use crate::net::{DataLinkProtocol, MacAddr};
use core::task::Waker;
use foundation::io::{ReadError, WriteError};

pub trait Device: Send + Sync + 'static {
    fn mac_addr(&self) -> MacAddr;

    fn protocol(&self) -> DataLinkProtocol;

    fn wake_when_read_available(&mut self, waker: &Waker) {
        waker.wake_by_ref(); // immediately wake by default
    }

    fn wake_when_write_available(&mut self, waker: &Waker) {
        waker.wake_by_ref(); // immediately wake by default
    }

    fn read_frame(&mut self, buffer: &mut [u8]) -> Result<ReadFrameResult, ReadError>;

    fn write_frame(&mut self, buffer: &[u8]) -> Result<usize, WriteError>;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ReadFrameResult {
    Incomplete(usize),
    Complete(usize),
}
