use crate::net::serialize::{WireSerializable, WireSerializer};
use core::cmp::PartialEq;
use core::iter::repeat_n;
use foundation::io::{Write, WriteExactError};
use thiserror::Error;
use foundation::net::MacAddr;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct EthernetFrame<'a> {
    mac_destination: MacAddr,
    mac_source: MacAddr,
    ether_type: EtherType,
    payload: &'a [u8],
    fcs: [u8; 4],
}

impl<T> WireSerializable<T> for EthernetFrame<'_>
where
    T: Write<u8>,
{
    fn serialize(&self, s: &mut WireSerializer<T>) -> Result<(), WriteExactError> {
        s.write_raw(&self.mac_destination.octets())?;
        s.write_raw(&self.mac_source.octets())?;
        // TODO: support qtag
        s.write_raw(Into::<[u8; 2]>::into(self.ether_type))?;
        s.write_raw(self.payload)?;
        const MIN_PAYLOAD_LEN: usize = 46; // TODO: change this to 42 once we support Q-Tags
        s.write_from(repeat_n(
            0,
            MIN_PAYLOAD_LEN.saturating_sub(self.payload.len()),
        ))?;
        s.write_raw(&self.fcs)?;
        Ok(())
    }
}

impl<'a> EthernetFrame<'a> {
    pub fn new(
        mac_destination: MacAddr,
        mac_source: MacAddr,
        ether_type: EtherType,
        payload: &'a [u8],
    ) -> Self {
        Self {
            mac_destination,
            mac_source,
            ether_type,
            payload,
            fcs: [0; 4], // TODO: compute checksum here
        }
    }

    pub fn ether_type(&self) -> EtherType {
        self.ether_type
    }

    pub fn payload(&self) -> &'a [u8] {
        self.payload
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Error)]
pub enum InvalidEthernetFrame {
    #[error("frame too short: expected {expected} bytes, got {actual}")]
    TooShort { expected: usize, actual: usize },
    #[error("invalid ether type")]
    InvalidEtherType,
    #[error("invalid frame check sequence")]
    ChecksumError,
}

impl<'a> TryFrom<&'a [u8]> for EthernetFrame<'a> {
    type Error = InvalidEthernetFrame;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        const MIN_LENGTH: usize = 64;

        if value.len() < MIN_LENGTH {
            return Err(InvalidEthernetFrame::TooShort {
                expected: MIN_LENGTH,
                actual: value.len(),
            });
        }

        let mac_destination = MacAddr::from(TryInto::<[u8; 6]>::try_into(&value[0..6]).unwrap());
        let mac_source = MacAddr::from(TryInto::<[u8; 6]>::try_into(&value[6..12]).unwrap());
        // TODO: support qtag
        let ethertype = (&value[12..14])
            .try_into()
            .map_err(|_| InvalidEthernetFrame::InvalidEtherType)?;
        let payload = &value[14..value.len() - 4];
        let fcs: [u8; 4] = (&value[value.len() - 4..]).try_into().unwrap();

        // TODO: check frame check sequence

        Ok(Self {
            mac_destination,
            mac_source,
            ether_type: ethertype,
            payload,
            fcs,
        })
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum EtherType {
    Ipv4,
    Arp,
}

impl From<EtherType> for [u8; 2] {
    fn from(value: EtherType) -> Self {
        match value {
            EtherType::Ipv4 => [0x08, 0x00],
            EtherType::Arp => [0x08, 0x06],
        }
    }
}

impl TryFrom<&[u8]> for EtherType {
    type Error = ();

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let arr: [u8; 2] = value.try_into().map_err(|_| ())?;
        Self::try_from(arr)
    }
}

impl TryFrom<[u8; 2]> for EtherType {
    type Error = ();

    fn try_from(value: [u8; 2]) -> Result<Self, Self::Error> {
        match value {
            [0x08, 0x00] => Ok(Self::Ipv4),
            [0x08, 0x06] => Ok(Self::Arp),
            _ => Err(()),
        }
    }
}
