#![no_std]
#![feature(allocator_api)]
extern crate alloc;

use crate::net::{Arp, Device, Interface, Ip, Route, RoutingTable};
use alloc::boxed::Box;
use alloc::sync::Arc;
use foundation::future::executor::{block_on, Executor, Tick, TickResult};
use foundation::net::IpCidr;
use protocols::Protocols;

mod net;
mod protocols;

pub struct NetStack {
    executor: Executor<'static>,
    routing: Arc<RoutingTable>,
    protocols: Arc<Protocols>,
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
        let protocols = Arc::new(Protocols::new(
            Arp::new(routing.clone()),
            Ip::new(routing.clone()),
        ));

        Self {
            executor,
            routing,
            protocols,
        }
    }

    pub fn arp(&self) -> &Arp {
        &self.protocols.arp()
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
