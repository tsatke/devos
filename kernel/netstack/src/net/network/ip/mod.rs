use crate::net::ethernet::{EtherType, EthernetFrame};
use crate::net::serialize::WireSerializer;
use crate::net::{DataLinkProtocol, Frame, Interface, RoutingTable};
use alloc::sync::Arc;
use core::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use core::task::Waker;
use crossbeam::queue::SegQueue;
use foundation::falloc::vec::FVec;
pub use header::*;
pub use packet::*;
use thiserror::Error;
use foundation::net::MacAddr;

mod checksum;
mod header;
mod packet;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Error)]
pub enum IpSendError {
    #[error("no interface configured to serve the outgoing ip address")]
    NoOutgoingInterface,
}

pub struct Ip {
    routing: Arc<RoutingTable>,
    wakers: SegQueue<Waker>,
}

impl Ip {
    pub fn new(routing: Arc<RoutingTable>) -> Self {
        Self {
            routing,
            wakers: SegQueue::new(),
        }
    }

    pub async fn send_data(
        &self,
        source_protocol: IpHeaderProtocol,
        destination: IpAddr,
        data: &[u8],
    ) -> Result<(), IpSendError> {
        let interface = self
            .routing
            .interface_that_serves_ip(destination)
            .await
            .ok_or(IpSendError::NoOutgoingInterface)?;

        match destination {
            IpAddr::V4(v4) => self.send_data4(source_protocol, interface, v4, data).await,
            IpAddr::V6(v6) => self.send_data6(interface, v6, data).await,
        }
    }

    async fn send_data4(
        &self,
        source_protocol: IpHeaderProtocol,
        interface: Arc<Interface>,
        destination: Ipv4Addr,
        data: &[u8],
    ) -> Result<(), IpSendError> {
        let destination_mac = MacAddr::BROADCAST; // FIXME: translate destination ip with arp

        let (our_ip, our_mac) = {
            let guard = interface.addresses().lock().await;
            (
                guard.ipv4_addr().unwrap_or(Ipv4Addr::UNSPECIFIED),
                guard.mac_addr(),
            )
        };
        let ip_header = IpHeader::new_v4(
            source_protocol,
            our_ip,
            destination,
            64,                // TODO: better ttl
            data.len() as u16, // TODO: check whether we need to fragment, otherwise this could panic
        );

        let packet = IpPacket::new(ip_header, data);
        let packet_len = packet.header().header_length() + packet.header().payload_length();
        let mut packet_raw = FVec::try_with_capacity(packet_len).unwrap();
        WireSerializer::new(&mut packet_raw)
            .write_serializable(&packet)
            .unwrap();

        let ethernet_frame =
            EthernetFrame::new(destination_mac, our_mac, EtherType::Ipv4, &packet_raw); // FIXME: fragmentation for ethernet

        let mut ethernet_raw = FVec::try_with_capacity(18 + packet_len).unwrap();
        WireSerializer::new(&mut ethernet_raw)
            .write_serializable(&ethernet_frame)
            .unwrap();

        interface
            .send_frame(Frame::new(DataLinkProtocol::Ethernet, ethernet_raw))
            .await;
        Ok(())
    }

    async fn send_data6(
        &self,
        interface: Arc<Interface>,
        destination: Ipv6Addr,
        data: &[u8],
    ) -> Result<(), IpSendError> {
        todo!()
    }

    pub async fn process_packet(&self, packet: IpPacket<'_>) {
        todo!()
    }
}
