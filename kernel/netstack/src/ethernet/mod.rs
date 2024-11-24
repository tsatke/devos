use crate::arp::{Arp, ArpReceiveError};
use crate::ip::{Ip, IpReceiveError};
use crate::{Netstack, Protocol};
use alloc::sync::Arc;
use futures::future::BoxFuture;
use futures::FutureExt;
use thiserror::Error;

use crate::interface::Interface;
pub use frame::*;

mod frame;

pub struct Ethernet(Arc<Netstack>);

impl Ethernet {
    pub(crate) fn new(netstack: Arc<Netstack>) -> Self {
        Self(netstack)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Error)]
pub enum EthernetReceiveError {
    #[error("error reading frame")]
    ReadFrame(#[from] ReadEthernetFrameError),
    #[error("error handling ip packet")]
    Ip(#[from] IpReceiveError),
    #[error("error handling arp packet")]
    Arp(#[from] ArpReceiveError),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Error)]
pub enum EthernetSendError {}

impl Protocol for Ethernet {
    type Packet<'packet> = EthernetFrame<'packet>;
    type ReceiveError = EthernetReceiveError;
    type SendError = EthernetSendError;

    fn name() -> &'static str {
        "ethernet"
    }

    fn receive_packet<'a>(
        &self,
        interface: Arc<Interface>,
        packet: Self::Packet<'a>,
    ) -> BoxFuture<'a, Result<(), Self::ReceiveError>> {
        let net = self.0.clone();
        async move {
            match packet.ether_type {
                EtherType::Ipv4 => {
                    net.handle_incoming_packet::<Ip, _>(interface, packet)
                        .await?
                }
                EtherType::Arp => {
                    net.handle_incoming_packet::<Arp, _>(interface, packet)
                        .await?
                }
            };
            Ok(())
        }
        .boxed()
    }

    fn send_packet(&self, _packet: Self::Packet<'_>) -> BoxFuture<Result<(), Self::SendError>> {
        todo!()
    }
}
