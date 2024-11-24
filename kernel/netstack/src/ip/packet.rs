use crate::ethernet::EthernetFrame;
use core::net::Ipv4Addr;
use thiserror::Error;

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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Ipv4HeaderFlags {
    pub reserved: bool,
    pub dont_fragment: bool,
    pub more_fragments: bool,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Ipv4Protocol {
    Udp,
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
