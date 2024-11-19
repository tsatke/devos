use crate::net::network::ip::header::IpHeader;
use crate::net::serialize::{WireSerializable, WireSerializer};
use crate::net::InvalidIpHeader;
use derive_more::Constructor;
use foundation::io::{Write, WriteExactError};
use thiserror::Error;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Constructor)]
pub struct IpPacket<'a> {
    header: IpHeader,
    payload: &'a [u8],
}

impl<'a> IpPacket<'a> {
    pub fn header(&self) -> IpHeader {
        self.header
    }

    pub fn payload(&self) -> &'a [u8] {
        self.payload
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Error)]
pub enum InvalidIpPacket {
    #[error("invalid header")]
    InvalidHeader(#[from] InvalidIpHeader),
    #[error("payload too short, header specified more data")]
    PayloadTooShort,
}

impl<'a> TryFrom<&'a [u8]> for IpPacket<'a> {
    type Error = InvalidIpPacket;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        let header = IpHeader::try_from(value)?;
        let header_length = header.header_length();
        let payload_length = header.payload_length();
        if value.len() < header_length + payload_length {
            return Err(InvalidIpPacket::PayloadTooShort);
        }
        let payload = &value[header_length..header_length + payload_length];
        Ok(Self { header, payload })
    }
}

impl<T> WireSerializable<T> for IpPacket<'_>
where
    T: Write<u8>,
{
    fn serialize(&self, s: &mut WireSerializer<T>) -> Result<(), WriteExactError> {
        s.write_serializable(&self.header)?;
        s.write_raw(self.payload)?;
        Ok(())
    }
}
