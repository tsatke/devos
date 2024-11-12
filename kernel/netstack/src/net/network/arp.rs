use crate::net::MacAddr;
use alloc::collections::BTreeMap;
use core::net::Ipv4Addr;
use core::task::Waker;
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

impl ArpPacket {
    pub fn htype(&self) -> u16 {
        self.htype
    }

    pub fn ptype(&self) -> u16 {
        self.ptype
    }

    pub fn hlen(&self) -> u8 {
        self.hlen
    }

    pub fn plen(&self) -> u8 {
        self.plen
    }

    pub fn operation(&self) -> ArpOperation {
        self.operation
    }

    pub fn srchaddr(&self) -> MacAddr {
        self.srchaddr
    }

    pub fn srcpaddr(&self) -> [u8; 4] {
        self.srcpaddr
    }

    pub fn dsthaddr(&self) -> MacAddr {
        self.dsthaddr
    }

    pub fn dstpaddr(&self) -> [u8; 4] {
        self.dstpaddr
    }
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

        self.cache.lock().await.insert(ip, packet.dsthaddr);

        while let Some(waker) = self.wakers.pop() {
            waker.wake();
        }
    }
}