use crate::net::phy::Physical;
use alloc::boxed::Box;

pub struct Interface {
    device: Box<dyn Physical>,
}
