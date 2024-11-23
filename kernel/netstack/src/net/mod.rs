// we want to reexport this, because we use this instead of our own wrappers
#[allow(unused_imports)]
pub use core::net::{IpAddr, Ipv4Addr, Ipv6Addr};
pub use link::interface::*;
pub use link::*;
pub use network::*;
pub use phy::*;
pub use routing::*;

mod link;
mod network;
mod phy;
mod routing;
mod serialize;
