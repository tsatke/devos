use crate::udp::{Udp, UdpError};
use crate::{Netstack, Protocol};
use alloc::sync::Arc;
use derive_more::Constructor;
use futures::future::BoxFuture;
use futures::FutureExt;
use thiserror::Error;

pub use packet::*;

mod packet;

#[derive(Constructor)]
pub struct Ip(Arc<Netstack>);

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
        packet: Self::Packet<'a>,
    ) -> BoxFuture<'a, Result<(), Self::Error>> {
        let net = self.0.clone();
        async move {
            match packet {
                IpPacket::V4 { protocol, .. } => match protocol {
                    Ipv4Protocol::Udp => net.handle_packet::<Udp, _>(packet).await?,
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
