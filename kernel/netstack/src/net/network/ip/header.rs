use crate::net::network::ip::checksum::Checksum;
use crate::net::serialize::{WireSerializable, WireSerializer};
use core::net::{Ipv4Addr, Ipv6Addr};
use foundation::io::{Write, WriteExactError};
use thiserror::Error;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum IpHeader {
    V4 {
        ihl: u8,
        dscp: u8,
        ecn: u8,
        total_length: u16,
        identification: u16,
        flags: u8,
        fragment_offset: u16,
        ttl: u8,
        protocol: u8,
        header_checksum: u16,
        source_address: Ipv4Addr,
        destination_address: Ipv4Addr,
    },
    V6 {
        traffic_class: u8,
        flow_label: u32,
        payload_length: u16,
        next_header: u8,
        hop_limit: u8,
        source_address: Ipv6Addr,
        destination_address: Ipv6Addr,
    },
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Error)]
pub enum InvalidIpHeader {
    #[error("packet too short")]
    TooShort,
    #[error("invalid ip version")]
    InvalidVersion,
    #[error("invalid header length")]
    InvalidHeaderLength,
    #[error("invalid checksum")]
    InvalidChecksum,
    #[error("invalid source address")]
    InvalidSourceAddress,
    #[error("invalid destination address")]
    InvalidDestinationAddress,
    #[error("invalid protocol")]
    InvalidProtocol,
    #[error("invalid total length")]
    InvalidTotalLength,
}

impl<'a> TryFrom<&'a [u8]> for IpHeader {
    type Error = InvalidIpHeader;
    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        const MIN_LENGTH: usize = 16;

        if data.len() < MIN_LENGTH {
            return Err(InvalidIpHeader::TooShort);
        }

        let version = data[0] >> 4;

        match version {
            4 => Ok(Self::try_parse4(data)?),
            6 => Ok(Self::try_parse6(data)?),
            _ => Err(InvalidIpHeader::InvalidVersion),
        }
    }
}

pub enum IpHeaderProtocol {
    Icmp,
    Ipmg,
    Tcp,
    Udp,
    Encap,
    Ospf,
    Sctp,
}

impl IpHeaderProtocol {
    pub fn value(&self) -> u8 {
        match self {
            IpHeaderProtocol::Icmp => 1,
            IpHeaderProtocol::Ipmg => 2,
            IpHeaderProtocol::Tcp => 6,
            IpHeaderProtocol::Udp => 17,
            IpHeaderProtocol::Encap => 41,
            IpHeaderProtocol::Ospf => 89,
            IpHeaderProtocol::Sctp => 132,
        }
    }
}

impl IpHeader {
    pub fn new_v4(
        protocol: IpHeaderProtocol,
        source_address: Ipv4Addr,
        destination_address: Ipv4Addr,
        ttl: u8,
        payload_len: u16,
    ) -> Self {
        let ihl = 5;
        let header_len = ihl * 4;
        assert!(payload_len <= u16::MAX - header_len);
        let mut header = Self::V4 {
            ihl: 5, // TODO: options, which makes this field larger
            dscp: 0,
            ecn: 0,
            total_length: header_len + payload_len,
            identification: 0,
            flags: 0,
            fragment_offset: 0,
            ttl,
            protocol: protocol.value(),
            header_checksum: 0,
            source_address,
            destination_address,
        };
        let header_checksum = header.compute_checksum();
        *header.header_checksum_mut().unwrap() = header_checksum;
        header
    }

    fn compute_checksum(&self) -> u16 {
        let mut checksum = Checksum::default();
        WireSerializer::new(&mut checksum)
            .write_serializable(self)
            .unwrap();
        checksum.finalize()
    }

    fn checksum_valid(&self) -> bool {
        self.compute_checksum() == 0
    }

    pub fn header_length(&self) -> usize {
        match *self {
            IpHeader::V4 { ihl, .. } => ihl as usize * 4,
            IpHeader::V6 { .. } => 40, // TODO: extension headers
        }
    }

    pub fn payload_length(&self) -> usize {
        match *self {
            IpHeader::V4 { total_length, .. } => total_length as usize - self.header_length(),
            IpHeader::V6 { payload_length, .. } => payload_length as usize,
        }
    }

    fn header_checksum_mut(&mut self) -> Option<&mut u16> {
        match self {
            IpHeader::V4 {
                header_checksum, ..
            } => Some(header_checksum),
            _ => None,
        }
    }

    fn try_parse4(data: &[u8]) -> Result<Self, InvalidIpHeader> {
        let ihl = data[0] & 0x0F;
        if ihl < 5 {
            return Err(InvalidIpHeader::InvalidHeaderLength);
        }

        let dscp = data[1] >> 2;
        let ecn = data[1] & 0x03;
        let total_length = u16::from_be_bytes([data[2], data[3]]);
        let identification = u16::from_be_bytes([data[4], data[5]]);
        let flags = data[6] >> 5;
        let fragment_offset = u16::from_be_bytes([data[6] & 0x1F, data[7]]);
        let ttl = data[8];
        let protocol = data[9];
        let header_checksum = u16::from_be_bytes([data[10], data[11]]);
        let source_address =
            Ipv4Addr::from(u32::from_be_bytes([data[12], data[13], data[14], data[15]]));
        let destination_address =
            Ipv4Addr::from(u32::from_be_bytes([data[16], data[17], data[18], data[19]]));

        let header = Self::V4 {
            ihl,
            dscp,
            ecn,
            total_length,
            identification,
            flags,
            fragment_offset,
            ttl,
            protocol,
            header_checksum,
            source_address,
            destination_address,
        };
        if !header.checksum_valid() {
            return Err(InvalidIpHeader::InvalidChecksum);
        }
        Ok(header)
    }

    fn try_parse6(data: &[u8]) -> Result<Self, InvalidIpHeader> {
        let traffic_class = data[0] & 0x0F | data[1] >> 4;
        let flow_label = u32::from_be_bytes([0, data[1] & 0x0F, data[2], data[3]]);
        let payload_length = u16::from_be_bytes([data[4], data[5]]);
        let next_header = data[6]; // TODO: actually parse next_header and extension headers
        let hop_limit = data[7];
        let source_address = Ipv6Addr::from([
            u16::from_be_bytes([data[8], data[9]]),
            u16::from_be_bytes([data[10], data[11]]),
            u16::from_be_bytes([data[12], data[13]]),
            u16::from_be_bytes([data[14], data[15]]),
            u16::from_be_bytes([data[16], data[17]]),
            u16::from_be_bytes([data[18], data[19]]),
            u16::from_be_bytes([data[20], data[21]]),
            u16::from_be_bytes([data[22], data[23]]),
        ]);
        let destination_address = Ipv6Addr::from([
            u16::from_be_bytes([data[24], data[25]]),
            u16::from_be_bytes([data[26], data[27]]),
            u16::from_be_bytes([data[28], data[29]]),
            u16::from_be_bytes([data[30], data[31]]),
            u16::from_be_bytes([data[32], data[33]]),
            u16::from_be_bytes([data[34], data[35]]),
            u16::from_be_bytes([data[36], data[37]]),
            u16::from_be_bytes([data[38], data[39]]),
        ]);

        Ok(Self::V6 {
            traffic_class,
            flow_label,
            payload_length,
            next_header,
            hop_limit,
            source_address,
            destination_address,
        })
    }
}

impl<T> WireSerializable<T> for IpHeader
where
    T: Write<u8>,
{
    fn serialize(&self, s: &mut WireSerializer<T>) -> Result<(), WriteExactError> {
        match *self {
            IpHeader::V4 {
                ihl,
                dscp,
                ecn,
                total_length,
                identification,
                flags,
                fragment_offset,
                ttl,
                protocol,
                header_checksum,
                source_address,
                destination_address,
            } => {
                s.write_u8((4 << 4) | ihl)?;
                s.write_u8((dscp << 2) | ecn)?;
                s.write_u16(total_length)?;
                s.write_u16(identification)?;
                s.write_u16((u16::from(flags) << 5) | (fragment_offset >> 8))?;
                s.write_u8(ttl)?;
                s.write_u8(protocol)?;
                s.write_u16(header_checksum)?;
                s.write_raw(source_address.octets())?;
                s.write_raw(destination_address.octets())?;
            }
            IpHeader::V6 {
                traffic_class,
                flow_label,
                payload_length,
                next_header,
                hop_limit,
                source_address,
                destination_address,
            } => {
                s.write_u8((6 << 4) | (traffic_class >> 4))?;
                s.write_u8((traffic_class << 4) | (flow_label >> 16) as u8)?;
                s.write_u16(flow_label as u16)?;
                s.write_u16(payload_length)?;
                s.write_u8(next_header)?;
                s.write_u8(hop_limit)?;
                s.write_raw(source_address.octets())?;
                s.write_raw(destination_address.octets())?;
            }
        }
        Ok(())
    }
}
