#![no_std]
#![feature(allocator_api)]
extern crate alloc;

use crate::executor::{ExecuteResult, Executor};
use crate::net::ethernet::{EtherType, EthernetFrame};
use crate::net::{Arp, ArpPacket, DataLinkProtocol, Device, Frame, Interface, IpCidr};
use alloc::boxed::Box;
use alloc::sync::Arc;
use derive_more::From;
use foundation::falloc::vec::FVec;
use futures::StreamExt;
use spin::Mutex;

mod async_queue;
pub mod executor;
mod net;

pub struct NetStack {
    executor: Executor,
    routing: Mutex<FVec<Route>>,
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
        let frame_stream = interface.frames().map_err(|_| ())?;

        let route = Route(cidr, interface);
        self.routing.lock().try_push(route).map_err(|_| ())?;

        self.executor.spawn({
            let mut frame_stream = frame_stream.fuse();
            let protocols = self.protocols.clone();
            async move {
                while let Some(frame) = frame_stream.next().await {
                    match protocols.route_link_frame(frame).await {
                        Ok(_) => {}
                        Err(_) => panic!("failed to route packet"), // FIXME: just log an error and continue
                    }
                }
                // no more frames will come from this device, so we can exit the task
            }
        })?;

        Ok(())
    }

    pub fn execute_step(&self) -> ExecuteResult {
        self.executor.execute_task()
    }
}

#[derive(From)]
pub struct Route(IpCidr, Interface);
