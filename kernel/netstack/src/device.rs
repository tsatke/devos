use crate::net::{DataLinkProtocol, MacAddr};
use core::task::Waker;
use foundation::io::{Read, Write};

pub trait Device: Read<u8> + Write<u8> + Send + Sync + 'static {
    fn mac_addr(&self) -> MacAddr;
    fn protocol(&self) -> DataLinkProtocol;
    fn wake_upon_data_available(&mut self, waker: &Waker) {
        waker.wake_by_ref(); // immediately wake by default
    }
}
