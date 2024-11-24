use crate::arp::{Arp, ArpError};
use crate::ip::{Ip, IpError};
use crate::{Netstack, Protocol};
use alloc::sync::Arc;
use derive_more::Constructor;
use foundation::falloc::vec::FVec;
use foundation::io::{Write, WriteExactError, WriteInto};
use foundation::net::MacAddr;
use futures::future::BoxFuture;
use futures::FutureExt;
use thiserror::Error;

#[derive(Constructor)]
pub struct Ethernet(Arc<Netstack>);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Error)]
pub enum EthernetError {
    #[error("error reading frame")]
    ReadFrame(#[from] ReadEthernetFrameError),
    #[error("error handling ip packet")]
    Ip(#[from] IpError),
    #[error("error handling arp packet")]
    Arp(#[from] ArpError),
}

impl Protocol for Ethernet {
    type Packet<'packet> = EthernetFrame<'packet>;
    type Error = EthernetError;

    fn process_packet<'a>(
        &self,
        packet: Self::Packet<'a>,
    ) -> BoxFuture<'a, Result<(), Self::Error>> {
        let net = self.0.clone();
        async move {
            match packet.ether_type {
                EtherType::Ipv4 => net.handle_packet::<Ip, _>(packet).await?,
                EtherType::Arp => net.handle_packet::<Arp, _>(packet).await?,
            };
            Ok(())
        }
        .boxed()
    }

    fn send_packet(&self, _packet: Self::Packet<'_>) -> BoxFuture<Result<(), Self::Error>> {
        todo!()
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct RawEthernetFrame {
    data: FVec<u8>,
}

impl AsRef<[u8]> for RawEthernetFrame {
    fn as_ref(&self) -> &[u8] {
        &self.data
    }
}

pub enum EtherType {
    Ipv4,
    Arp,
}

pub struct Qtag {
    pub tpid: u16,
    pub tci: u16,
}

impl Qtag {
    pub fn size(&self) -> usize {
        4
    }
}

/// A valid ethernet frame.
pub struct EthernetFrame<'a> {
    pub mac_destination: MacAddr,
    pub mac_source: MacAddr,
    pub qtag: Option<Qtag>,
    pub ether_type: EtherType,
    pub payload: &'a [u8],
}

impl EthernetFrame<'_> {
    pub fn size(&self) -> usize {
        18 + self.qtag.as_ref().map_or(0, Qtag::size) + self.payload.len().max(64)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Error)]
pub enum ReadEthernetFrameError {
    #[error("frame too short: expected {expected} bytes, got {actual}")]
    TooShort { expected: usize, actual: usize },
    #[error("invalid ether type")]
    InvalidEtherType,
    #[error("invalid frame check sequence")]
    ChecksumError,
}

impl<'frame, 'raw> TryFrom<&'raw [u8]> for EthernetFrame<'frame> {
    type Error = ReadEthernetFrameError;

    fn try_from(_value: &'raw [u8]) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl TryFrom<RawEthernetFrame> for EthernetFrame<'_> {
    type Error = ReadEthernetFrameError;

    fn try_from(value: RawEthernetFrame) -> Result<Self, Self::Error> {
        TryFrom::<&[u8]>::try_from(value.as_ref())
    }
}

impl WriteInto<u8> for EthernetFrame<'_> {
    fn write_into(&self, _out: &mut impl Write<u8>) -> Result<(), WriteExactError> {
        todo!()
    }
}
