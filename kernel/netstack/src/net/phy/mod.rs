use crate::net::MacAddr;
pub use interface::*;

mod interface;

pub trait Physical {
    fn mac_addr(&self) -> MacAddr;
    fn read(&mut self, buffer: &mut [u8]) -> Result<usize, ()>;
    fn write(&mut self, buffer: &[u8]) -> Result<usize, ()>;
}
