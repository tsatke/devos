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
pub enum UdpReceiveError {
    #[error("failed to read udp packet")]
    ReadPacket(#[from] ReadUdpPacketError),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Error)]
pub enum UdpSendError {}

impl Protocol for Udp {
    type Packet<'packet> = UdpDatagram<'packet>;
    type ReceiveError = UdpReceiveError;
    type SendError = UdpSendError;

    fn name() -> &'static str {
        "udp"
    }

    fn receive_packet<'a>(
        &self,
        _interface: Arc<Interface>,
        _packet: Self::Packet<'a>,
    ) -> BoxFuture<'a, Result<(), Self::ReceiveError>> {
        todo!()
    }

    fn send_packet<'a>(
        &self,
        _packet: Self::Packet<'a>,
    ) -> BoxFuture<'a, Result<(), Self::SendError>> {
        todo!()
    }
}
