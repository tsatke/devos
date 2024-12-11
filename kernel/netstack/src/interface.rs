use crate::device::RawDataLinkFrame;
use alloc::sync::Arc;
use core::fmt::{Debug, Formatter};
use core::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use foundation::future::lock::{FutureMutex, Spin};
use foundation::future::queue::AsyncBoundedQueue;
use foundation::net::{Ipv4Cidr, Ipv6Cidr, MacAddr};

pub struct Interface {
    mac_addr: MacAddr,
    rx_queue: Arc<AsyncBoundedQueue<RawDataLinkFrame>>,
    tx_queue: Arc<AsyncBoundedQueue<RawDataLinkFrame>>,
    addresses: FutureMutex<Config>,
}

#[derive(Debug, Default, Eq, PartialEq)]
pub struct Config {
    ipv4addr: Option<Ipv4Addr>,
    ipv4cidr: Option<Ipv4Cidr>,
    ipv6addr: Option<Ipv6Addr>,
    ipv6cidr: Option<Ipv6Cidr>,
}

impl Debug for Interface {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Interface")
            .field("mac_addr", &self.mac_addr)
            .field("addresses", &self.addresses.lock_sync::<Spin>())
            .finish_non_exhaustive()
    }
}

impl Interface {
    pub fn new(
        mac_addr: MacAddr,
        rx_queue: Arc<AsyncBoundedQueue<RawDataLinkFrame>>,
        tx_queue: Arc<AsyncBoundedQueue<RawDataLinkFrame>>,
    ) -> Self {
        Self {
            mac_addr,
            rx_queue,
            tx_queue,
            addresses: FutureMutex::default(),
        }
    }

    pub fn mac_address(&self) -> MacAddr {
        self.mac_addr
    }

    pub async fn ipv4_addr(&self) -> Option<Ipv4Addr> {
        self.addresses.lock().await.ipv4addr
    }

    pub async fn set_ipv4_addr(&self, addr: Ipv4Addr) {
        self.addresses.lock().await.ipv4addr = Some(addr);
    }

    pub async fn should_serve(&self, ip: IpAddr) -> bool {
        let guard = self.addresses.lock().await;
        match ip {
            IpAddr::V4(v4) => guard.ipv4cidr.is_some_and(|cidr| cidr.contains(v4)),
            IpAddr::V6(v6) => guard.ipv6cidr.is_some_and(|cidr| cidr.contains(v6)),
        }
    }

    pub fn rx_queue(&self) -> &Arc<AsyncBoundedQueue<RawDataLinkFrame>> {
        &self.rx_queue
    }

    pub fn tx_queue(&self) -> &Arc<AsyncBoundedQueue<RawDataLinkFrame>> {
        &self.tx_queue
    }
}
