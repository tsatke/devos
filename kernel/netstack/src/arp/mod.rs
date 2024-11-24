use crate::{Netstack, Packet, Protocol};
use alloc::sync::Arc;
use core::net::Ipv4Addr;
use futures::future::BoxFuture;
use futures::FutureExt;
pub use packet::*;
use thiserror::Error;

use crate::ethernet::{EtherType, EthernetFrame, EthernetSendError};
use crate::interface::Interface;
pub use cache::*;
use foundation::falloc::vec::FVec;
use foundation::io::{Cursor, WriteInto};
use foundation::net::MacAddr;

mod cache;
mod packet;

#[derive(Clone)]
pub struct Arp(Arc<Netstack>);

impl Arp {
    pub(crate) fn new(netstack: Arc<Netstack>) -> Self {
        Self(netstack)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Error)]
pub enum ArpReceiveError {
    #[error("error reading packet")]
    ReadPacket(#[from] ReadArpPacketError),
    #[error("error sending packet")]
    Send(#[from] ArpSendError),
    #[error("out of memory")]
    AllocError,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Error)]
pub enum ArpSendError {
    #[error("error sending ethernet frame")]
    Ethernet(#[from] EthernetSendError),
    #[error("out of memory")]
    AllocError,
}

impl Protocol for Arp {
    type Packet<'packet> = ArpPacket;
    type ReceiveError = ArpReceiveError;
    type SendError = ArpSendError;

    fn name() -> &'static str {
        "arp"
    }

    fn receive_packet<'a>(
        &self,
        interface: Arc<Interface>,
        packet: Self::Packet<'a>,
    ) -> BoxFuture<'a, Result<(), Self::ReceiveError>> {
        let arp = self.clone();
        async move {
            match packet {
                ArpPacket::Ipv4Ethernet {
                    operation,
                    mac_destination,
                    mac_source,
                    ip_destination,
                    ip_source,
                } => {
                    arp.process_ipv4_ethernet(
                        interface,
                        operation,
                        mac_destination,
                        mac_source,
                        ip_destination,
                        ip_source,
                    )
                    .await
                }
            }
        }
        .boxed()
    }

    fn send_packet(&self, packet: Self::Packet<'_>) -> BoxFuture<Result<(), Self::SendError>> {
        async move {
            let mut raw = FVec::try_with_capacity(packet.wire_size())
                .map_err(|_| ArpSendError::AllocError)?;
            packet
                .write_into(Cursor::new(&mut raw))
                .map_err(|_| ArpSendError::AllocError)?;

            match packet {
                ArpPacket::Ipv4Ethernet {
                    mac_destination,
                    mac_source,
                    ..
                } => {
                    let frame = EthernetFrame::try_new(
                        mac_destination,
                        mac_source,
                        None,
                        EtherType::Arp,
                        &raw,
                    )
                    .expect("arp has only 28 bytes of payload, which must be small enough for an ethernet frame");
                    self.0.ethernet().send_packet(frame).await?;
                }
            };

            Ok(())
        }
        .boxed()
    }
}

impl Arp {
    async fn process_ipv4_ethernet(
        &self,
        interface: Arc<Interface>,
        operation: ArpOperation,
        mac_destination: MacAddr,
        mac_source: MacAddr,
        ip_destination: Ipv4Addr,
        ip_source: Ipv4Addr,
    ) -> Result<(), ArpReceiveError> {
        // get the mac and ip that we need to insert into the cache
        let (mac, ip) = match operation {
            ArpOperation::Request => (mac_source, ip_source),
            ArpOperation::Reply => (mac_destination, ip_destination),
        };
        self.0.arp_state.lock().await.insert(ip, mac);

        let our_mac = interface.device().mac_address();

        if operation != ArpOperation::Request
            || !(mac_destination.is_broadcast() || our_mac == mac_destination)
        {
            return Ok(()); // no need to reply
        }

        let our_ip = interface.ipv4_addr().await;
        if our_ip.is_none() {
            return Ok(()); // we can't reply because we don't have an IP yet
        }
        let our_ip = our_ip.unwrap();

        let reply_ip_destination = if ip_source.is_unspecified() {
            Ipv4Addr::BROADCAST
        } else {
            ip_source
        };

        if !interface.should_serve(reply_ip_destination.into()).await {
            return Ok(());
        }

        let reply = ArpPacket::Ipv4Ethernet {
            operation: ArpOperation::Reply,
            mac_destination: mac_source,
            mac_source: our_mac,
            ip_destination: reply_ip_destination,
            ip_source: our_ip,
        };
        self.send_packet(reply).await?;
        Ok(())
    }
}
