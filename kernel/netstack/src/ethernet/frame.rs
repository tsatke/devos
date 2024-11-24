use foundation::falloc::vec::FVec;
use foundation::io::{Write, WriteExactError, WriteInto};
use foundation::net::MacAddr;
use num_enum::{IntoPrimitive, TryFromPrimitive};
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
#[derive(Debug, PartialEq)]
pub struct EthernetFrame<'a> {
    pub mac_destination: MacAddr,
    pub mac_source: MacAddr,
    pub qtag: Option<Qtag>,
    pub ether_type: EtherType,
    pub payload: &'a [u8],
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u16)]
pub enum EtherType {
    Ipv4 = 0x0800,
    Arp = 0x0806,
}

#[derive(Debug, Eq, PartialEq)]
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
    #[error("invalid ether type: {0:04x}")]
    InvalidEtherType(u16),
    #[error("invalid frame check sequence")]
    ChecksumError,
}

impl<'raw> TryFrom<&'raw [u8]> for EthernetFrame<'raw> {
    type Error = ReadEthernetFrameError;

    fn try_from(value: &'raw [u8]) -> Result<Self, Self::Error> {
        const MIN_LENGTH: usize = 64;
        if value.len() < MIN_LENGTH {
            return Err(ReadEthernetFrameError::TooShort {
                expected: MIN_LENGTH,
                actual: value.len(),
            });
        }

        let mac_destination =
            MacAddr::from([value[0], value[1], value[2], value[3], value[4], value[5]]);
        let mac_source =
            MacAddr::from([value[6], value[7], value[8], value[9], value[10], value[11]]);
        let ether_type = u16::from_be_bytes([value[12], value[13]]);
        let (payload_start, ether_type, qtag) = if ether_type == 0x8100 {
            let tpid = ether_type;
            let tci = u16::from_be_bytes([value[14], value[15]]);
            let qtag = Qtag { tpid, tci };
            let ether_type = u16::from_be_bytes([value[16], value[17]]);
            (18, ether_type, Some(qtag))
        } else {
            (14, ether_type, None)
        };
        let ether_type = EtherType::try_from(ether_type)
            .map_err(|e| ReadEthernetFrameError::InvalidEtherType(e.number))?;

        let payload = &value[payload_start..value.len() - 4];

        // TODO: validate FCS
        let _fcs = u32::from_be_bytes([
            value[value.len() - 4],
            value[value.len() - 3],
            value[value.len() - 2],
            value[value.len() - 1],
        ]);

        Ok(Self {
            mac_destination,
            mac_source,
            qtag,
            ether_type,
            payload,
        })
    }
}

impl<'a> TryFrom<&'a RawEthernetFrame> for EthernetFrame<'a> {
    type Error = ReadEthernetFrameError;

    fn try_from(value: &'a RawEthernetFrame) -> Result<Self, Self::Error> {
        TryFrom::<&'a [u8]>::try_from(value.as_ref())
    }
}

impl WriteInto<u8> for EthernetFrame<'_> {
    fn write_into(&self, out: &mut impl Write<u8>) -> Result<(), WriteExactError> {
        out.write_exact(self.mac_destination.octets().as_slice())?;
        out.write_exact(self.mac_source.octets().as_slice())?;
        if let Some(qtag) = self.qtag.as_ref() {
            out.write_exact(&qtag.tpid.to_be_bytes())?;
            out.write_exact(&qtag.tci.to_be_bytes())?;
        }
        out.write_exact(&Into::<u16>::into(self.ether_type).to_be_bytes())?;
        out.write_exact(self.payload)?;
        // TODO: padding

        let qtag_size = self.qtag.as_ref().map_or(0, Qtag::size);
        let padding_size = (46 - qtag_size).saturating_sub(self.payload.len());
        for _ in 0..padding_size {
            out.write_exact(&[0])?; // TODO: make more efficient?
        }

        // TODO: compute fcs
        out.write_exact(&[0, 0, 0, 0])?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;
    use foundation::io::Cursor;

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

    #[test]
    fn test_serialize_deserialize_qtag() {
        let frame = EthernetFrame {
            mac_destination: MacAddr::from([0x01, 0x23, 0x45, 0x67, 0x89, 0xAB]),
            mac_source: MacAddr::from([0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC]),
            qtag: Some(Qtag {
                tpid: 0x8100,
                tci: 0x1234,
            }),
            ether_type: EtherType::Ipv4,
            payload: [0xAB; 400].as_slice(),
        };
        let mut buf = Vec::new();
        let mut cursor = Cursor::new(&mut buf);
        frame.write_into(&mut cursor).unwrap();
        let frame2 = EthernetFrame::try_from(buf.as_slice()).unwrap();
        assert_eq!(frame, frame2);
    }

    #[test]
    fn test_serialize_deserialize() {
        let frame = EthernetFrame {
            mac_destination: MacAddr::from([0x01, 0x23, 0x45, 0x67, 0x89, 0xAB]),
            mac_source: MacAddr::from([0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC]),
            qtag: None,
            ether_type: EtherType::Ipv4,
            payload: [0xAB; 400].as_slice(),
        };
        let mut buf = Vec::new();
        let mut cursor = Cursor::new(&mut buf);
        frame.write_into(&mut cursor).unwrap();
        let frame2 = EthernetFrame::try_from(buf.as_slice()).unwrap();
        assert_eq!(frame, frame2);
    }
}
