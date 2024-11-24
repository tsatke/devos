use crate::ethernet::EthernetFrame;
use crate::{Netstack, Protocol};
use alloc::sync::Arc;
use core::net::Ipv4Addr;
use derive_more::Constructor;
use futures::future::BoxFuture;
use thiserror::Error;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Ipv4Protocol {}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Ipv4HeaderFlags {
    pub reserved: bool,
    pub dont_fragment: bool,
    pub more_fragments: bool,
}

pub enum IpPacket<'a> {
    V4 {
        header_length: u8,
        dscp: u8,
        ecn: u8,
        total_length: u16,
        identification: u16,
        flags: Ipv4HeaderFlags,
        fragment_offset: u16,
        time_to_live: u8,
        protocol: Ipv4Protocol,
        source: Ipv4Addr,
        destination: Ipv4Addr,
        payload: &'a [u8],
    },
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Error)]
pub enum ReadIpPacketError {}

impl<'a> TryFrom<&'a [u8]> for IpPacket<'a> {
    type Error = ReadIpPacketError;

    fn try_from(_value: &'a [u8]) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl<'a> TryFrom<EthernetFrame<'a>> for IpPacket<'a> {
    type Error = ReadIpPacketError;

    fn try_from(value: EthernetFrame<'a>) -> Result<Self, Self::Error> {
        TryFrom::<&'a [u8]>::try_from(value.payload)
    }
}

#[derive(Constructor)]
pub struct Ip(Arc<Netstack>);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Error)]
pub enum IpError {
    #[error("error reading packet")]
    ReadPacket(#[from] ReadIpPacketError),
}

impl Protocol for Ip {
    type Packet<'packet> = IpPacket<'packet>;
    type Error = IpError;

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
