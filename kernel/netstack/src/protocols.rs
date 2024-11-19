use crate::net::ethernet::{EtherType, EthernetFrame, InvalidEthernetFrame};
use crate::net::{
    Arp, ArpPacket, DataLinkProtocol, Frame, InvalidArpPacket, InvalidIpPacket, Ip, IpPacket,
};
use derive_more::Constructor;
use thiserror::Error;

#[derive(Constructor)]
pub struct Protocols {
    arp: Arp,
    ip: Ip,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Error)]
pub enum NoRoute {
    #[error("invalid frame")]
    InvalidFrame(#[from] InvalidFrame),
    #[error("protocol error")]
    ProtocolError(#[from] ProtocolError),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Error)]
pub enum InvalidFrame {
    #[error("invalid ethernet frame")]
    Ethernet(#[from] InvalidEthernetFrame),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Error)]
pub enum ProtocolError {
    #[error("invalid packet")]
    InvalidPacket(#[from] InvalidPacket),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Error)]
pub enum InvalidPacket {
    #[error("invalid arp packet")]
    Arp(#[from] InvalidArpPacket),
    #[error("invalid ip packet")]
    Ip(#[from] InvalidIpPacket),
}

impl Protocols {
    pub async fn route_link_frame(&self, frame: Frame) -> Result<(), NoRoute> {
        let protocol = frame.protocol();
        let data = frame.into_data();
        Ok(match protocol {
            DataLinkProtocol::Ethernet => {
                let frame =
                    EthernetFrame::try_from(data.as_ref()).map_err(InvalidFrame::Ethernet)?;
                self.route_ethernet_frame(&frame).await?
            }
        })
    }

    async fn route_ethernet_frame(&self, frame: &EthernetFrame<'_>) -> Result<(), ProtocolError> {
        match frame.ether_type() {
            EtherType::Ipv4 => {
                let packet = IpPacket::try_from(frame.payload()).map_err(InvalidPacket::Ip)?;
                self.ip.process_packet(packet).await;
            }
            EtherType::Arp => {
                let packet = ArpPacket::try_from(frame.payload()).map_err(InvalidPacket::Arp)?;
                self.arp.process_packet(packet).await;
            }
        }
        Ok(())
    }

    pub fn arp(&self) -> &Arp {
        &self.arp
    }

    pub fn ip(&self) -> &Ip {
        &self.ip
    }
}
