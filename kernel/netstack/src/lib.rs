#![no_std]
#![feature(allocator_api)]
extern crate alloc;

use crate::net::ethernet::{EtherType, EthernetFrame};
use crate::net::{Arp, ArpPacket, DataLinkProtocol, Device, Frame, Interface, IpCidr};
use alloc::boxed::Box;
use alloc::sync::Arc;
use core::net::IpAddr;
use derive_more::From;
use foundation::falloc::vec::FVec;
use foundation::future::executor::{block_on, ExecuteResult, Executor};
use foundation::future::lock::FutureMutex;

mod net;

pub struct NetStack {
    executor: Executor<'static>,
    routing: FutureMutex<FVec<Route>>,
    protocols: Arc<Protocols>,
}

struct Protocols {
    arp: Arp,
}

impl Protocols {
    async fn route_link_frame(&self, frame: Frame) -> Result<(), ()> {
        let protocol = frame.protocol();
        let data = frame.into_data();
        match protocol {
            DataLinkProtocol::Ethernet => {
                let frame = EthernetFrame::try_from(data.as_ref())?;
                self.route_ethernet_frame(&frame).await
            }
        }
    }

    async fn route_ethernet_frame(&self, frame: &EthernetFrame<'_>) -> Result<(), ()> {
        match frame.ether_type() {
            EtherType::Ipv4 => {
                todo!()
            }
            EtherType::Arp => {
                let packet = ArpPacket::try_from(frame.payload())?;
                self.arp.process_packet(packet).await;
            }
        }
        Ok(())
    }
}

impl NetStack {
    pub fn register_device(&self, cidr: IpCidr, device: Box<dyn Device>) -> Result<(), ()> {
        let interface = Interface::new(device);

        self.executor.spawn(interface.work_rx_queue());
        self.executor.spawn(interface.work_tx_queue());

        let rx = interface.rx_queue().clone();

        let route = Route(cidr, interface);
        block_on(async move {
            let mut guard = self.routing.lock().await;
            guard.try_push(route)
        }).map_err(|_| ())?;

        self.executor.spawn({
            let protocols = self.protocols.clone();
            async move {
                loop {
                    let frame = rx.pop().await;
                    match protocols.route_link_frame(frame).await {
                        Ok(_) => {}
                        Err(_) => panic!("failed to route packet"), // FIXME: just log an error and continue
                    }
                }
            }
        });

        Ok(())
    }

    pub fn execute_step(&self) -> ExecuteResult {
        self.executor.execute_task()
    }
}

#[derive(From)]
pub struct Route(IpCidr, Interface); // TODO: probably allow more CIDRs

impl Route {
    pub fn should_serve(&self, ip: IpAddr) -> bool {
        self.0.contains(ip).unwrap_or(false)
    }

    pub fn interface(&self) -> &Interface {
        &self.1
    }
}
