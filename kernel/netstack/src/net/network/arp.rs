use crate::net::ethernet::{EtherType, EthernetFrame};
use crate::net::serialize::{WireSerializable, WireSerializer};
use crate::net::{DataLinkProtocol, Frame, MacAddr, RoutingTable};
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use core::future::poll_fn;
use core::net::Ipv4Addr;
use core::task::{Poll, Waker};
use crossbeam::queue::SegQueue;
use derive_more::Constructor;
use foundation::falloc::vec::FVec;
use foundation::future::lock::FutureMutex;
use foundation::io::{Cursor, Write, WriteExactError};
use thiserror::Error;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ArpOperation {
    Request,
    Reply,
}

#[derive(Constructor, Debug, Copy, Clone, Eq, PartialEq)]
pub struct ArpPacket {
    htype: u16,
    ptype: u16,
    hlen: u8,
    plen: u8,
    operation: ArpOperation,
    srchaddr: MacAddr,
    srcpaddr: Ipv4Addr,
    dsthaddr: MacAddr,
    dstpaddr: Ipv4Addr,
}

impl<T> WireSerializable<T> for ArpPacket
where
    T: Write<u8>,
{
    fn serialize(&self, s: &mut WireSerializer<T>) -> Result<(), WriteExactError> {
        s.write_u16(self.htype)?;
        s.write_u16(self.ptype)?;
        s.write_u8(self.hlen)?;
        s.write_u8(self.plen)?;
        s.write_u16(match self.operation {
            ArpOperation::Request => 1,
            ArpOperation::Reply => 2,
        })?;
        s.write_raw(&self.srchaddr.octets())?;
        s.write_raw(&self.srcpaddr.octets())?;
        s.write_raw(&self.dsthaddr.octets())?;
        s.write_raw(&self.dstpaddr.octets())?;
        Ok(())
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Error)]
pub enum InvalidArpPacket {
    #[error("packet too short")]
    TooShort,
    #[error("invalid arp operation")]
    InvalidOperation,
}

impl TryFrom<&[u8]> for ArpPacket {
    type Error = InvalidArpPacket;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        // packet could be padded (if received through ethernet for example), so we just look
        // at the first 28 bytes
        if value.len() < 28 {
            return Err(InvalidArpPacket::TooShort);
        }

        let htype = u16::from_be_bytes([value[0], value[1]]);
        let ptype = u16::from_be_bytes([value[2], value[3]]);
        let hlen = value[4];
        let plen = value[5];
        let operation = match u16::from_be_bytes([value[6], value[7]]) {
            1 => ArpOperation::Request,
            2 => ArpOperation::Reply,
            _ => return Err(InvalidArpPacket::InvalidOperation),
        };
        let srchaddr = MacAddr::from(TryInto::<[u8; 6]>::try_into(&value[8..14]).unwrap());
        let srcpaddr = Ipv4Addr::from(TryInto::<[u8; 4]>::try_into(&value[14..18]).unwrap());
        let dsthaddr = MacAddr::from(TryInto::<[u8; 6]>::try_into(&value[18..24]).unwrap());
        let dstpaddr = Ipv4Addr::from(TryInto::<[u8; 4]>::try_into(&value[24..28]).unwrap());

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
    routing: Arc<RoutingTable>,
    wakers: SegQueue<Waker>,
    cache: FutureMutex<BTreeMap<Ipv4Addr, MacAddr>>,
}

impl Arp {
    pub fn new(routing: Arc<RoutingTable>) -> Self {
        Self {
            routing,
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
            todo!("actually send an arp request");
            Poll::Pending
        })
        .await
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

        let mac = packet.srchaddr;
        let ip = Ipv4Addr::from(packet.srcpaddr);

        if !(ip.is_broadcast() || mac.is_broadcast()) {
            self.cache.lock().await.insert(ip, mac);

            while let Some(waker) = self.wakers.pop() {
                waker.wake();
            }
        }

        if packet.operation == ArpOperation::Request {
            let sender_ip = packet.srcpaddr;
            let interface = self
                .routing
                .interface_that_serves_ip(sender_ip.into())
                .await
                .expect("should have interface");

            let sender_mac = packet.srchaddr;

            let addresses = *interface.addresses().lock().await;
            let our_ip = addresses.ipv4_addr().unwrap_or(Ipv4Addr::UNSPECIFIED);

            if interface.protocol() != DataLinkProtocol::Ethernet {
                todo!("only ethernet interfaces supported for now");
            }

            let our_mac = addresses.mac_addr();
            let arp_packet = ArpPacket::new(
                1,      // ethernet
                0x0800, // ipv4
                6,      // mac address length
                4,      // ipv4 address length
                ArpOperation::Reply,
                our_mac,
                our_ip,
                sender_mac,
                sender_ip,
            );

            let mut arp_raw = FVec::try_with_capacity(28).unwrap(); // TODO: handle error
            WireSerializer::new(Cursor::new(&mut arp_raw))
                .write_serializable(arp_packet)
                .unwrap();

            // we know that the interface is an ethernet interface
            let ethernet_frame = EthernetFrame::new(sender_mac, our_mac, EtherType::Arp, &arp_raw);

            let mut ethernet_raw = FVec::try_with_capacity(68).unwrap(); // TODO: handle error
            WireSerializer::new(Cursor::new(&mut ethernet_raw))
                .write_serializable(ethernet_frame)
                .unwrap();

            let frame = Frame::new(DataLinkProtocol::Ethernet, ethernet_raw);
            interface.send_frame(frame).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::ethernet::{EtherType, EthernetFrame};
    use crate::net::phy::testing::TestDevice;
    use crate::net::{DataLinkProtocol, IpCidr};
    use crate::NetStack;
    use alloc::boxed::Box;
    use alloc::vec::Vec;
    use foundation::future::executor::{block_on, Tick};
    use foundation::io::Cursor;

    #[test]
    fn test_arp_translate_broadcast() {
        let arp = Arp::new(Arc::new(RoutingTable::new()));
        assert_eq!(
            Some(MacAddr::BROADCAST),
            block_on(arp.translate(Ipv4Addr::BROADCAST))
        );
    }

    #[test]
    fn test_translate_cache_hit_from_request() {
        let net = NetStack::new();
        let arp = net.arp();

        let cidr = IpCidr::new_v4(Ipv4Addr::new(192, 168, 1, 0), 24).unwrap();
        let receiver_ipv4 = Ipv4Addr::new(192, 168, 1, 1);
        let receiver_mac = MacAddr::new([0xBE, 0xEF, 0xDE, 0xAD, 0xC0, 0xDE]);
        let sender_ipv4 = Ipv4Addr::new(192, 168, 1, 2);
        let sender_mac = MacAddr::new([0xF0, 0x0B, 0xA4, 0x5E, 0x12, 0x34]);

        let device = Box::new(TestDevice::create(receiver_mac, DataLinkProtocol::Ethernet));
        let rx_queue = device.rx_queue.clone();
        let tx_queue = device.tx_queue.clone();

        net.add_device(cidr.into(), device);
        block_on({
            let net = &net;
            async move {
                let interface = net
                    .routing
                    .interface_that_serves_ip(sender_ipv4.into())
                    .await
                    .expect("should have an interface");
                let mut guard = interface.addresses().lock().await;
                guard.set_ipv4_addr(Some(receiver_ipv4));
            }
        });

        let mut arp_raw = Vec::<u8>::new();
        WireSerializer::new(Cursor::new(&mut arp_raw))
            .write_serializable(ArpPacket::new(
                1,      // ethernet
                0x0800, // ipv4
                6,      // mac address length
                4,      // ipv4 address length
                ArpOperation::Request,
                sender_mac,
                sender_ipv4,
                MacAddr::BROADCAST,
                Ipv4Addr::BROADCAST,
            ))
            .unwrap();
        let mut ethernet_raw = Vec::<u8>::new();
        WireSerializer::new(Cursor::new(&mut ethernet_raw))
            .write_serializable(EthernetFrame::new(
                MacAddr::BROADCAST,
                sender_mac,
                EtherType::Arp,
                &arp_raw,
            ))
            .unwrap();

        rx_queue.push_now(ethernet_raw).unwrap();

        for _ in 0..10 {
            net.tick();
        }

        assert_eq!(Some(sender_mac), block_on(arp.translate(sender_ipv4)));

        let arp_reply = tx_queue.pop_now().expect("a reply should have been sent");
        let arp_reply = EthernetFrame::try_from(arp_reply.as_ref()).unwrap();
        let arp_reply = ArpPacket::try_from(arp_reply.payload()).unwrap();
        assert_eq!(ArpOperation::Reply, arp_reply.operation);
        assert_eq!(receiver_mac, arp_reply.srchaddr);
        assert_eq!(receiver_ipv4, arp_reply.srcpaddr);
        assert_eq!(sender_mac, arp_reply.dsthaddr);
        assert_eq!(sender_ipv4, arp_reply.dstpaddr);
        assert_eq!(1, arp_reply.htype);
        assert_eq!(0x0800, arp_reply.ptype);
        assert_eq!(6, arp_reply.hlen);
        assert_eq!(4, arp_reply.plen);
    }
}
