use crate::udp::{Udp, UdpError};
use crate::{Netstack, Protocol};
use alloc::sync::Arc;
use futures::future::BoxFuture;
use futures::FutureExt;
use thiserror::Error;

use crate::interface::Interface;
pub use packet::*;

mod packet;

pub struct Ip(Arc<Netstack>);

impl Ip {
    pub(crate) fn new(netstack: Arc<Netstack>) -> Self {
        Self(netstack)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Error)]
pub enum IpError {
    #[error("error reading packet")]
    ReadPacket(#[from] ReadIpPacketError),
    #[error("error handling udp packet")]
    Udp(#[from] UdpError),
}

impl Protocol for Ip {
    type Packet<'packet> = IpPacket<'packet>;
    type Error = IpError;

    fn name() -> &'static str {
        "ip"
    }

    fn process_packet<'a>(
        &self,
        interface: Arc<Interface>,
        packet: Self::Packet<'a>,
    ) -> BoxFuture<'a, Result<(), Self::Error>> {
        let net = self.0.clone();
        async move {
            match packet {
                IpPacket::V4 { protocol, .. } => match protocol {
                    Ipv4Protocol::Udp => {
                        net.handle_incoming_packet::<Udp, _>(interface, packet)
                            .await?
                    }
                },
            }
            Ok(())
        }
        .boxed()
    }

    fn send_packet(&self, _packet: Self::Packet<'_>) -> BoxFuture<Result<(), Self::Error>> {
        todo!()
    }
}
