use foundation::falloc::vec::FVec;
use foundation::io::{Write, WriteExactError, WriteInto};
use foundation::net::MacAddr;
use thiserror::Error;

#[derive(Debug, Eq, PartialEq)]
pub struct RawEthernetFrame {
    data: FVec<u8>,
}

impl AsRef<[u8]> for RawEthernetFrame {
    fn as_ref(&self) -> &[u8] {
        &self.data
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

impl EthernetFrame<'_> {
    pub fn size(&self) -> usize {
        let qtag = self.qtag.as_ref().map_or(0, Qtag::size);
        6 + // mac_destination
            6 + // mac_source
            qtag + // qtag
            2 + // ether_type
            self.payload.len().max(46 - qtag) + // payload
            4 // fcs
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

impl<'raw> TryFrom<&'raw [u8]> for EthernetFrame<'_> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size() {
        for (payload, size) in [
            ([].as_slice(), 64),
            ([0].as_slice(), 64),
            ([1; 17].as_slice(), 64),
            ([2; 46].as_slice(), 64),
            ([0xAB; 47].as_slice(), 65),
        ] {
            let frame = EthernetFrame {
                mac_destination: MacAddr::BROADCAST,
                mac_source: MacAddr::BROADCAST,
                qtag: None,
                ether_type: EtherType::Ipv4,
                payload,
            };
            assert_eq!(
                frame.size(),
                size,
                "expected size {} for payload {:?}, but got {}",
                size,
                payload,
                frame.size(),
            );
        }
    }
}
