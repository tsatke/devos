use crate::{Netstack, Protocol};
use alloc::sync::Arc;
use derive_more::Constructor;
use futures::future::BoxFuture;
use thiserror::Error;

pub use datagram::*;

mod datagram;

#[derive(Constructor)]
pub struct Udp(Arc<Netstack>);

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
        _packet: Self::Packet<'a>,
    ) -> BoxFuture<'a, Result<(), Self::Error>> {
        todo!()
    }

    fn send_packet(&self, _packet: Self::Packet<'_>) -> BoxFuture<Result<(), Self::Error>> {
        todo!()
    }
}
