use crate::{Netstack, Protocol};
use alloc::sync::Arc;
use derive_more::Constructor;
use futures::future::BoxFuture;
use thiserror::Error;

pub use packet::*;

mod packet;

#[derive(Constructor)]
pub struct Arp(Arc<Netstack>);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Error)]
pub enum ArpError {
    #[error("error reading packet")]
    ReadPacket(#[from] ReadArpPacketError),
}

impl Protocol for Arp {
    type Packet<'packet> = ArpPacket;
    type Error = ArpError;

    fn name() -> &'static str {
        "arp"
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
