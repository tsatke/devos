use crate::{Netstack, Protocol};
use alloc::sync::Arc;
use derive_more::Constructor;
use futures::future::BoxFuture;
use thiserror::Error;

pub use packet::*;

mod packet;

#[derive(Constructor)]
pub struct Arp(Arc<Netstack>);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Error)]
pub enum ArpError {
    #[error("error reading packet")]
    ReadPacket(#[from] ReadArpPacketError),
}

impl Protocol for Arp {
    type Packet<'packet> = ArpPacket;
    type Error = ArpError;

    fn name() -> &'static str {
        "arp"
    }

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

#[cfg(test)]
mod tests {
    use crate::arp::packet::{ArpOperation, ArpPacket, ReadArpPacketError};
    use alloc::vec::Vec;
    use core::net::Ipv4Addr;
    use foundation::io::{Cursor, WriteInto};
    use foundation::net::MacAddr;

    #[test]
    fn test_arp_packet_serialize_deserialize() {
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
    fn test_arp_packet_deserialize_invalids() {
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
    fn test_arp_packet_deserialize_padded() {
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
