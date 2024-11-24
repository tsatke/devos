use crate::ethernet::EthernetFrame;
use core::net::Ipv4Addr;
use foundation::io::{Write, WriteExactError, WriteInto};
use foundation::net::MacAddr;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use thiserror::Error;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ArpPacket {
    Ipv4Ethernet {
        operation: ArpOperation,
        mac_destination: MacAddr,
        mac_source: MacAddr,
        ip_destination: Ipv4Addr,
        ip_source: Ipv4Addr,
    },
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u16)]
pub enum ArpOperation {
    Request = 1,
    Reply = 2,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u16)]
pub enum HardwareType {
    Ethernet = 0x0001,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u16)]
pub enum ProtocolType {
    Ipv4 = 0x0800,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Error)]
pub enum ReadArpPacketError {
    #[error("packet too short: expected {expected}, actual {actual}")]
    TooShort { expected: usize, actual: usize },
    #[error("unknown hlen: {0}")]
    UnknownHlen(u8),
    #[error("unknown plen: {0}")]
    UnknownPlen(u8),
    #[error("unknown operation: {0:04x}")]
    UnknownOperation(u16),
    #[error("unknown protocol type: {0:04x}")]
    UnknownProtocol(u16),
    #[error("unknown hardware type: {0:04x}")]
    UnknownHardware(u16),
}

impl<'a> TryFrom<&'a [u8]> for ArpPacket {
    type Error = ReadArpPacketError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        // we need at least 4 bytes to read the hardware type and protocol type
        if value.len() < 4 {
            return Err(ReadArpPacketError::TooShort {
                expected: 4,
                actual: value.len(),
            });
        }

        let hardware_type = u16::from_be_bytes([value[0], value[1]]);
        let hardware_type = HardwareType::try_from(hardware_type)
            .map_err(|e| ReadArpPacketError::UnknownHardware(e.number))?;

        let protocol_type = u16::from_be_bytes([value[2], value[3]]);
        let protocol_type = ProtocolType::try_from(protocol_type)
            .map_err(|e| ReadArpPacketError::UnknownProtocol(e.number))?;

        match (hardware_type, protocol_type) {
            (HardwareType::Ethernet, ProtocolType::Ipv4) => Self::decode_ipv4_ethernet(value),
        }
    }
}

impl ArpPacket {
    fn decode_ipv4_ethernet(value: &[u8]) -> Result<Self, ReadArpPacketError> {
        const MIN_LENGTH: usize = 28;

        // we need at least 28 bytes to read the rest of the packet
        if value.len() < MIN_LENGTH {
            return Err(ReadArpPacketError::TooShort {
                expected: MIN_LENGTH,
                actual: value.len(),
            });
        }

        let hlen = value[4];
        if hlen != 6 {
            return Err(ReadArpPacketError::UnknownHlen(hlen));
        }

        let plen = value[5];
        if plen != 4 {
            return Err(ReadArpPacketError::UnknownPlen(plen));
        }

        let operation = ArpOperation::try_from(u16::from_be_bytes([value[6], value[7]]))
            .map_err(|e| ReadArpPacketError::UnknownOperation(e.number))?;

        let mac_source = MacAddr::from([
            value[8], value[9], value[10], value[11], value[12], value[13],
        ]);
        let ip_source = Ipv4Addr::from([value[14], value[15], value[16], value[17]]);
        let mac_destination = MacAddr::from([
            value[18], value[19], value[20], value[21], value[22], value[23],
        ]);
        let ip_destination = Ipv4Addr::from([value[24], value[25], value[26], value[27]]);

        Ok(ArpPacket::Ipv4Ethernet {
            operation,
            mac_destination,
            mac_source,
            ip_destination,
            ip_source,
        })
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
        match self {
            ArpPacket::Ipv4Ethernet {
                operation,
                mac_destination,
                mac_source,
                ip_destination,
                ip_source,
            } => {
                out.write_exact(&Into::<u16>::into(HardwareType::Ethernet).to_be_bytes())?;
                out.write_exact(&Into::<u16>::into(ProtocolType::Ipv4).to_be_bytes())?;
                out.write_exact(&[6, 4])?; // hlen, plen
                out.write_exact(&Into::<u16>::into(*operation).to_be_bytes())?;
                out.write_exact(mac_source.octets().as_slice())?;
                out.write_exact(&ip_source.octets())?;
                out.write_exact(mac_destination.octets().as_slice())?;
                out.write_exact(&ip_destination.octets())?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::arp::packet::{ArpOperation, ArpPacket, ReadArpPacketError};
    use alloc::vec::Vec;
    use core::net::Ipv4Addr;
    use foundation::io::{Cursor, WriteInto};
    use foundation::net::MacAddr;

    #[test]
    fn test_serialize_deserialize() {
        let packet = ArpPacket::Ipv4Ethernet {
            operation: ArpOperation::Reply,
            mac_destination: MacAddr::from([1, 2, 3, 4, 5, 6]),
            mac_source: MacAddr::from([7, 8, 9, 10, 11, 12]),
            ip_destination: Ipv4Addr::from([192, 168, 1, 1]),
            ip_source: Ipv4Addr::from([192, 168, 1, 2]),
        };

        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);
        packet.write_into(&mut cursor).unwrap();

        let packet2 = ArpPacket::try_from(buffer.as_slice()).unwrap();
        assert_eq!(packet, packet2);
    }

    #[test]
    fn test_deserialize_invalids() {
        let data = [
            0x00_u8, 0x02, // hardware type (unsupported)
            0x08, 0x00, // protocol type
            0x06, 0x04, // hlen, plen
            0x00, 0x02, // operation
            1, 2, 3, 4, 5, 6, // mac source
            192, 168, 1, 1, // ip source
            7, 8, 9, 10, 11, 12, // mac destination
            192, 168, 1, 2, // ip destination
        ];
        let packet = ArpPacket::try_from(data.as_slice());
        assert_eq!(Err(ReadArpPacketError::UnknownHardware(0x0002)), packet);
    }

    #[test]
    fn test_deserialize_padded() {
        let data = [
            0x00_u8, 0x01, // hardware type
            0x08, 0x00, // protocol type
            0x06, 0x04, // hlen, plen
            0x00, 0x02, // operation
            1, 2, 3, 4, 5, 6, // mac source
            192, 168, 1, 1, // ip source
            7, 8, 9, 10, 11, 12, // mac destination
            192, 168, 1, 2, // ip destination
            0, 0, 0, 0, 0, 0, 0, 0, // padding
        ];
        let packet = ArpPacket::try_from(data.as_slice());
        assert_eq!(
            Ok(ArpPacket::Ipv4Ethernet {
                operation: ArpOperation::Reply,
                mac_destination: MacAddr::from([7, 8, 9, 10, 11, 12]),
                mac_source: MacAddr::from([1, 2, 3, 4, 5, 6]),
                ip_destination: Ipv4Addr::from([192, 168, 1, 2]),
                ip_source: Ipv4Addr::from([192, 168, 1, 1]),
            }),
            packet
        );
    }
}
