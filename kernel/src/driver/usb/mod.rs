mod device;
mod protocol_support;
mod speed;

pub use device::*;
pub use protocol_support::*;
pub use speed::*;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Protocol {
    USB2,
    USB3,
    USB3_1,
    USB3_2,
}
