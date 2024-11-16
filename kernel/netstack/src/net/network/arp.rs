use crate::net::MacAddr;
use alloc::collections::BTreeMap;
use core::future::poll_fn;
use core::net::Ipv4Addr;
use core::task::{Poll, Waker};
use crossbeam::queue::SegQueue;
use foundation::future::lock::FutureMutex;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ArpOperation {
    Request,
    Reply,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ArpPacket {
    htype: u16,
    ptype: u16,
    hlen: u8,
    plen: u8,
    operation: ArpOperation,
    srchaddr: MacAddr,
    srcpaddr: [u8; 4],
    dsthaddr: MacAddr,
    dstpaddr: [u8; 4],
}

impl TryFrom<&[u8]> for ArpPacket {
    type Error = ();

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < 28 {
            return Err(());
        }

        let htype = u16::from_be_bytes([value[0], value[1]]);
        let ptype = u16::from_be_bytes([value[2], value[3]]);
        let hlen = value[4];
        let plen = value[5];
        let operation = match u16::from_be_bytes([value[6], value[7]]) {
            1 => ArpOperation::Request,
            2 => ArpOperation::Reply,
            _ => return Err(()),
        };
        let srchaddr = MacAddr::from(TryInto::<[u8; 6]>::try_into(&value[8..14]).unwrap());
        let srcpaddr = TryInto::<[u8; 4]>::try_into(&value[14..18]).unwrap();
        let dsthaddr = MacAddr::from(TryInto::<[u8; 6]>::try_into(&value[18..24]).unwrap());
        let dstpaddr = TryInto::<[u8; 4]>::try_into(&value[24..28]).unwrap();

        Ok(Self {
            htype,
            ptype,
            hlen,
            plen,
            operation,
            srchaddr,
            srcpaddr,
            dsthaddr,
            dstpaddr,
        })
    }
}

pub struct Arp {
    wakers: SegQueue<Waker>,
    cache: FutureMutex<BTreeMap<Ipv4Addr, MacAddr>>,
}

impl Arp {
    pub fn new() -> Self {
        Self {
            wakers: SegQueue::new(),
            cache: FutureMutex::new(BTreeMap::new()),
        }
    }

    pub async fn translate(&self, ip: Ipv4Addr) -> Option<MacAddr> {
        if ip.is_broadcast() {
            return Some(MacAddr::BROADCAST);
        }

        if let Some(mac) = self.cache.lock().await.get(&ip) {
            return Some(*mac);
        }

        poll_fn(|cx| {
            self.wakers.push(cx.waker().clone());
            Poll::Pending
        }).await
    }

    pub async fn process_packet(&self, packet: ArpPacket) {
        if packet.ptype != 0x0800 {
            // ignore non-ipv4 packets
            return;
        }

        if packet.hlen != 6 {
            // we only know hardware addresses with 6 bytes
            return;
        }

        if packet.plen != 4 {
            // ipv4 packets (which we have because of the ptype) should have
            // an address length of 4, so this is an invalid packet
            return;
        }

        if packet.operation == ArpOperation::Request {
            todo!("reply to arp requests");
        }

        let mac = packet.srchaddr;
        let ip = Ipv4Addr::from(packet.srcpaddr);

        self.cache.lock().await.insert(ip, mac);

        while let Some(waker) = self.wakers.pop() {
            waker.wake();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use foundation::future::executor::block_on;

    #[test]
    fn test_arp_translate_broadcast() {
        let arp = Arp::new();
        assert_eq!(Some(MacAddr::BROADCAST), block_on(arp.translate(Ipv4Addr::BROADCAST)));
    }
}