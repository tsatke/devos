#![no_std]
#![feature(allocator_api)]
extern crate alloc;

use crate::net::ethernet::{EtherType, EthernetFrame, InvalidEthernetFrame};
use crate::net::{
    Arp, ArpPacket, DataLinkProtocol, Device, Frame, Interface, InvalidArpPacket, IpCidr,
    RoutingTable,
};
use alloc::boxed::Box;
use alloc::sync::Arc;
use foundation::future::executor::{block_on, Executor, Tick, TickResult};
use net::Route;
use thiserror::Error;

mod net;

pub struct NetStack {
    executor: Executor<'static>,
    routing: Arc<RoutingTable>,
    protocols: Arc<Protocols>,
}

struct Protocols {
    arp: Arp,
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
}

impl Protocols {
    async fn route_link_frame(&self, frame: Frame) -> Result<(), NoRoute> {
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
                todo!()
            }
            EtherType::Arp => {
                let packet = ArpPacket::try_from(frame.payload()).map_err(InvalidPacket::Arp)?;
                self.arp.process_packet(packet).await;
            }
        }
        Ok(())
    }
}

impl Default for NetStack {
    fn default() -> Self {
        Self::new()
    }
}

impl NetStack {
    pub fn new() -> Self {
        let executor = Executor::new();
        let routing = Arc::new(RoutingTable::new());
        let protocols = Arc::new(Protocols {
            arp: Arp::new(routing.clone()),
        });

        Self {
            executor,
            routing,
            protocols,
        }
    }

    pub fn arp(&self) -> &Arp {
        &self.protocols.arp
    }

    pub fn add_device(&self, cidr: IpCidr, device: Box<dyn Device>) {
        let interface = Interface::new(device);

        self.executor.spawn(interface.work_rx_queue());
        self.executor.spawn(interface.work_tx_queue());

        let rx = interface.rx_queue().clone();

        let interface = Arc::new(interface);
        let route = Route::new(cidr, interface);
        block_on(async move {
            self.routing.add_route(route).await;
        });

        self.executor.spawn({
            let protocols = self.protocols.clone();
            async move {
                loop {
                    let frame = rx.pop().await;
                    // FIXME: just log an error and continue
                    protocols.route_link_frame(frame).await.unwrap();
                }
            }
        });
    }
}

impl Tick for NetStack {
    fn tick(&self) -> TickResult {
        self.executor.tick()
    }
}
