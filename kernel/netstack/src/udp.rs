use crate::ip::IpPacket;
use crate::{Netstack, Protocol};
use alloc::sync::Arc;
use core::marker::PhantomData;
use derive_more::Constructor;
use futures::future::BoxFuture;
use thiserror::Error;

#[derive(Constructor)]
pub struct Udp(Arc<Netstack>);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Error)]
pub enum UdpError {
    #[error("failed to read udp packet")]
    ReadPacket(#[from] ReadUdpPacketError),
}

pub struct UdpPacket<'a> {
    _lifetime: PhantomData<&'a ()>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Error)]
pub enum ReadUdpPacketError {}

impl<'a> TryFrom<IpPacket<'a>> for UdpPacket<'a> {
    type Error = ReadUdpPacketError;

    fn try_from(_packet: IpPacket<'a>) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl Protocol for Udp {
    type Packet<'packet> = UdpPacket<'packet>;
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
