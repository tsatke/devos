use crate::net::MacAddr;
use core::cmp::PartialEq;

pub struct EthernetFrame<'a> {
    mac_destination: MacAddr,
    mac_source: MacAddr,
    ether_type: EtherType,
    payload: &'a [u8],
    fcs: [u8; 4],
}

impl<'a> EthernetFrame<'a> {
    pub fn mac_destination(&self) -> MacAddr {
        self.mac_destination
    }

    pub fn mac_source(&self) -> MacAddr {
        self.mac_source
    }

    pub fn ether_type(&self) -> EtherType {
        self.ether_type
    }

    pub fn payload(&self) -> &'a [u8] {
        self.payload
    }

    pub fn fcs(&self) -> [u8; 4] {
        self.fcs
    }
}

impl<'a> TryFrom<&'a [u8]> for EthernetFrame<'a> {
    type Error = ();

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        if value.len() < 64 {
            return Err(());
        }

        let mac_destination: [u8; 6] = (&value[0..6]).try_into().unwrap();
        let mac_source: [u8; 6] = (&value[6..12]).try_into().unwrap();
        // TODO: support qtag
        let ethertype: [u8; 2] = (&value[12..14]).try_into().unwrap();
        let payload = &value[14..value.len() - 4];
        let fcs: [u8; 4] = (&value[value.len() - 4..]).try_into().unwrap();

        // TODO: check frame check sequence

        Ok(Self {
            mac_destination: mac_destination.into(),
            mac_source: mac_source.into(),
            ether_type: ethertype.try_into()?,
            payload,
            fcs,
        })
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum EtherType {
    Ipv4,
    Arp,
}
impl TryFrom<[u8; 2]> for EtherType {
    type Error = ();

    fn try_from(value: [u8; 2]) -> Result<Self, Self::Error> {
        match value {
            [0x08, 0x00] => Ok(Self::Ipv4),
            [0x08, 0x06] => Ok(Self::Arp),
            _ => Err(()),
        }
    }
}
