use crate::Netstack;
use alloc::sync::Arc;
use foundation::falloc::vec::FVec;
use foundation::io::{Write, WriteExactError, WriteInto};
use foundation::net::MacAddr;
use thiserror::Error;

pub struct Ethernet(Arc<Netstack>);

#[derive(Debug, Eq, PartialEq)]
pub struct RawEthernetFrame {
    pub data: FVec<u8>,
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

impl<'raw, 'frame> TryFrom<&'raw [u8]> for EthernetFrame<'frame>
where
    'frame: 'raw,
{
    type Error = ReadEthernetFrameError;

    fn try_from(_value: &'raw [u8]) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl WriteInto<u8> for EthernetFrame<'_> {
    fn write_into(&self, _out: &mut impl Write<u8>) -> Result<(), WriteExactError> {
        todo!()
    }
}
