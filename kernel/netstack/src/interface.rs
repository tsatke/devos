use crate::device::Device;
use alloc::boxed::Box;
use core::fmt::{Debug, Formatter};
use core::net::{IpAddr, Ipv4Addr};
use foundation::future::lock::FutureMutex;
use foundation::net::MacAddr;

pub struct Interface {
    mac_addr: MacAddr,
    device: Box<dyn Device>,
    state: FutureMutex<State>,
}

pub struct State {
    ipv4addr: Option<Ipv4Addr>,
}

impl Debug for Interface {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Interface")
            .field("mac_addr", &self.mac_addr)
            .finish_non_exhaustive()
    }
}

impl Interface {
    pub fn new(device: Box<dyn Device>) -> Self {
        let mac_addr = device.mac_address();
        Self {
            mac_addr,
            device,
            state: FutureMutex::new(State { ipv4addr: None }),
        }
    }

    pub fn mac_address(&self) -> MacAddr {
        self.mac_addr
    }

    pub fn device(&self) -> &dyn Device {
        &*self.device
    }

    pub async fn ipv4_addr(&self) -> Option<Ipv4Addr> {
        self.state.lock().await.ipv4addr
    }

    pub async fn should_serve(&self, _ip: IpAddr) -> bool {
        true // TODO: rely on CIDRs once this somewhat works
    }
}
