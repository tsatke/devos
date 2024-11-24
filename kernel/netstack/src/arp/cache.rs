use alloc::collections::BTreeMap;
use core::net::Ipv4Addr;
use foundation::net::MacAddr;
use log::info;

#[derive(Debug, Default)]
pub struct ArpCache {
    cache: BTreeMap<Ipv4Addr, MacAddr>,
}

impl ArpCache {
    pub fn insert(&mut self, ip: Ipv4Addr, mac: MacAddr) {
        info!("new arp entry: {ip} -> {mac}");
        self.cache.insert(ip, mac);
    }

    pub fn lookup(&self, ip: Ipv4Addr) -> Option<MacAddr> {
        self.cache.get(&ip).copied()
    }
}
