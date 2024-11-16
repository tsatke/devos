use crate::net::{ArpPacket, Interface, IpCidr};
use alloc::sync::Arc;
use core::net::IpAddr;
use derive_more::From;
use foundation::falloc::vec::FVec;
use foundation::future::lock::FutureMutex;

pub struct RoutingTable {
    routes: FutureMutex<FVec<Route>>,
}

impl RoutingTable {
    pub fn new() -> Self {
        Self {
            routes: FutureMutex::new(FVec::new()),
        }
    }

    pub async fn interfaces_for_ip(&self, ip: IpAddr) -> FVec<Arc<Interface>> {
        let guard = self.routes.lock().await;
        let iter = guard
            .iter()
            .filter(|route| route.should_serve(ip))
            .map(Route::interface)
            .cloned();
        let mut v = FVec::new();
        v.try_extend(iter).unwrap(); // TODO: handle error
        v
    }

    pub async fn add_route(&self, route: Route) {
        self.routes.lock().await.try_push(route).unwrap(); // TODO: handle error
    }

    pub async fn send_arp_packet(&self, packet: ArpPacket) {
        todo!("send_arp_packet")
    }
}

#[derive(From, Debug)]
pub struct Route {
    cidrs: FVec<IpCidr>,
    interface: Arc<Interface>,
}

impl Route {
    // TODO: support more than one IpCidr
    pub fn new(cidr: IpCidr, interface: Arc<Interface>) -> Self {
        let mut cidrs = FVec::new();
        cidrs.try_push(cidr).unwrap(); // TODO: handle error
        Self { cidrs, interface }
    }

    pub fn should_serve(&self, ip: IpAddr) -> bool {
        self.cidrs
            .iter()
            .any(|cidr| cidr.contains(ip).unwrap_or(false))
    }

    pub fn interface(&self) -> &Arc<Interface> {
        &self.interface
    }
}
