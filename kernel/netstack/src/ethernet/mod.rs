use crate::arp::{Arp, ArpReceiveError};
use crate::device::RawDataLinkFrame;
use crate::interface::Interface;
use crate::ip::{Ip, IpReceiveError};
use crate::{Netstack, Packet, Protocol};
use alloc::sync::Arc;
use foundation::falloc::vec::FVec;
use foundation::io::{Cursor, WriteInto};
pub use frame::*;
use futures::future::BoxFuture;
use futures::FutureExt;
use thiserror::Error;

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
pub enum EthernetSendError {
    #[error("out of memory")]
    AllocError,
}

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

    fn send_packet<'a>(
        &self,
        packet: Self::Packet<'a>,
    ) -> BoxFuture<'a, Result<(), Self::SendError>> {
        // FIXME: find right interface, which will require some kind of target ip address
        let net = self.0.clone();
        async move {
            let mut raw = FVec::try_with_capacity(packet.wire_size())
                .map_err(|_| EthernetSendError::AllocError)?;
            packet.write_into(Cursor::new(&mut raw)).unwrap(); // TODO: handle error

            let frame = RawDataLinkFrame::Ethernet(RawEthernetFrame::new(raw));
            net.interfaces.lock().await[0].tx_queue().push(frame).await;
            Ok(())
        }
        .boxed()
    }
}
