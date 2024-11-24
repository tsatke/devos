use crate::{Netstack, Protocol};
use alloc::sync::Arc;
use futures::future::BoxFuture;
use thiserror::Error;

use crate::interface::Interface;
pub use datagram::*;

mod datagram;

pub struct Udp(Arc<Netstack>);

impl Udp {
    pub(crate) fn new(netstack: Arc<Netstack>) -> Self {
        Self(netstack)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Error)]
pub enum UdpError {
    #[error("failed to read udp packet")]
    ReadPacket(#[from] ReadUdpPacketError),
}

impl Protocol for Udp {
    type Packet<'packet> = UdpDatagram<'packet>;
    type Error = UdpError;

    fn name() -> &'static str {
        "udp"
    }

    fn process_packet<'a>(
        &self,
        _interface: Arc<Interface>,
        _packet: Self::Packet<'a>,
    ) -> BoxFuture<'a, Result<(), Self::Error>> {
        todo!()
    }

    fn send_packet(&self, _packet: Self::Packet<'_>) -> BoxFuture<Result<(), Self::Error>> {
        todo!()
    }
}
