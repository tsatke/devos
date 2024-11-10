use crate::async_queue::AsyncArrayQueue;
use crate::net::MacAddr;
use alloc::sync::Arc;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ArpPacket {
    htype: u16,
    ptype: u16,
    hlen: u8,
    plen: u8,
    operation: u16,
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

    pub fn operation(&self) -> u16 {
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
        let operation = u16::from_be_bytes([value[6], value[7]]);
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
    incoming_packets: Arc<AsyncArrayQueue<ArpPacket>>,
}

impl Arp {
    pub async fn receive_packet(&self, incoming_packet: ArpPacket) {
        self.incoming_packets.push(incoming_packet).await;
    }
}
