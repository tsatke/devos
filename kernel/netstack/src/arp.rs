use crate::ethernet::EthernetFrame;
use crate::{Netstack, Protocol};
use alloc::sync::Arc;
use core::net::IpAddr;
use derive_more::Constructor;
use foundation::io::{Write, WriteExactError, WriteInto};
use foundation::net::MacAddr;
use futures::FutureExt;
use thiserror::Error;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ArpType {
    Request,
    Reply,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ArpPacket {
    Ipv4Ethernet {
        mac_destination: MacAddr,
        mac_source: MacAddr,
        ip_destination: IpAddr,
        ip_source: IpAddr,
    },
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Error)]
pub enum ReadArpPacketError {
    #[error("packet too short: expected {expected}, actual {actual}")]
    TooShort { expected: usize, actual: usize },
    #[error("unknown protocol type: {0:02x}")]
    UnknownProtocol(u16),
    #[error("unknown hardware type: {0:02x}")]
    UnknownHardware(u16),
}

impl<'a> TryFrom<&'a [u8]> for ArpPacket {
    type Error = ReadArpPacketError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl<'a> TryFrom<EthernetFrame<'a>> for ArpPacket {
    type Error = ReadArpPacketError;

    fn try_from(value: EthernetFrame<'a>) -> Result<Self, Self::Error> {
        TryFrom::<&'a [u8]>::try_from(value.payload)
    }
}

impl WriteInto<u8> for ArpPacket {
    fn write_into(&self, out: &mut impl Write<u8>) -> Result<(), WriteExactError> {
        todo!()
    }
}

#[derive(Constructor)]
pub struct Arp(Arc<Netstack>);

impl Arp {
    async fn process_packet_internal(&self, packet: ArpPacket) -> Result<(), ArpError> {
        todo!()
    }

    async fn send_packet_internal(&self, packet: ArpPacket) -> Result<(), ArpError> {
        todo!()
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Error)]
pub enum ArpError {}

impl Protocol for Arp {
    type Packet<'packet> = ArpPacket;
    type Error = ArpError;

    fn process_packet(
        &self,
        packet: Self::Packet<'_>,
    ) -> futures::future::BoxFuture<Result<(), Self::Error>> {
        self.process_packet_internal(packet).boxed()
    }

    fn send_packet(
        &self,
        packet: Self::Packet<'_>,
    ) -> futures::future::BoxFuture<Result<(), Self::Error>> {
        self.send_packet_internal(packet).boxed()
    }
}
