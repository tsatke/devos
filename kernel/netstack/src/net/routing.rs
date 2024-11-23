use crate::net::Interface;
use alloc::sync::Arc;
use core::net::IpAddr;
use derive_more::From;
use foundation::falloc::vec::FVec;
use foundation::future::lock::FutureMutex;
use foundation::net::IpCidr;

pub struct RoutingTable {
    routes: FutureMutex<FVec<Route>>,
}

impl RoutingTable {
    pub fn new() -> Self {
        Self {
            routes: FutureMutex::new(FVec::new()),
        }
    }

    pub async fn interface_that_serves_ip(&self, ip: IpAddr) -> Option<Arc<Interface>> {
        self.routes
            .lock()
            .await
            .iter()
            .filter(|route| route.should_serve(ip))
            .map(Route::interface)
            .next()
            .cloned()
    }

    pub async fn add_route(&self, route: Route) {
        self.routes.lock().await.try_push(route).unwrap(); // TODO: handle error
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
