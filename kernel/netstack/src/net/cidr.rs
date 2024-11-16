use core::fmt::Formatter;
use core::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use derive_more::{Display, Error};

#[derive(Debug, Display, Error, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct InvalidNetworkLength;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum IpCidr {
    V4(Ipv4Cidr),
    V6(Ipv6Cidr),
}

impl IpCidr {
    pub fn contains(&self, ip: IpAddr) -> Result<bool, ()> {
        match (self, ip) {
            (IpCidr::V4(cidr), IpAddr::V4(ip)) => Ok(cidr.contains(ip)),
            (IpCidr::V6(cidr), IpAddr::V6(ip)) => Ok(cidr.contains(ip)),
            _ => Err(()),
        }
    }
}

impl Display for IpCidr {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            IpCidr::V4(cidr) => cidr.fmt(f),
            IpCidr::V6(cidr) => cidr.fmt(f),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Ipv4Cidr(Ipv4Addr, u8);

impl Display for Ipv4Cidr {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}/{}", self.0, self.1)
    }
}

impl Ipv4Cidr {
    const MAX_NET_LEN: u8 = 32;

    pub const fn try_new(ip: Ipv4Addr, network: u8) -> Result<Self, InvalidNetworkLength> {
        if network > Self::MAX_NET_LEN {
            return Err(InvalidNetworkLength);
        }
        Ok(Ipv4Cidr(ip, network))
    }

    pub fn netmask(&self) -> Ipv4Addr {
        if self.1 == 0 {
            return Ipv4Addr::new(0, 0, 0, 0);
        }

        let mask = u32::MAX << (Self::MAX_NET_LEN - self.1);
        Ipv4Addr::from_bits(mask)
    }

    pub fn contains(&self, ip: Ipv4Addr) -> bool {
        self.0.to_bits() & self.netmask().to_bits() == ip.to_bits() & self.netmask().to_bits()
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Ipv6Cidr(Ipv6Addr, u8);

impl Display for Ipv6Cidr {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}/{}", self.0, self.1)
    }
}

impl Ipv6Cidr {
    const MAX_NET_LEN: u8 = 128;

    pub const fn try_new(ip: Ipv6Addr, network: u8) -> Result<Self, InvalidNetworkLength> {
        if network > Self::MAX_NET_LEN {
            return Err(InvalidNetworkLength);
        }
        Ok(Ipv6Cidr(ip, network))
    }

    pub fn netmask(&self) -> Ipv6Addr {
        if self.1 == 0 {
            return Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0);
        }

        let mask = u128::MAX << (Self::MAX_NET_LEN - self.1);
        Ipv6Addr::from_bits(mask)
    }

    pub fn contains(&self, ip: Ipv6Addr) -> bool {
        self.0.to_bits() & self.netmask().to_bits() == ip.to_bits() & self.netmask().to_bits()
    }
}

#[cfg(test)]
mod tests {
    use crate::net::{Ipv4Cidr, Ipv6Cidr};
    use core::net::{Ipv4Addr, Ipv6Addr};

    #[test]
    fn test_ipv4_cidr_new() {
        for network in 0..=Ipv4Cidr::MAX_NET_LEN {
            assert!(Ipv4Cidr::try_new(Ipv4Addr::new(0, 0, 0, 0), network).is_ok());
        }
        for network in Ipv4Cidr::MAX_NET_LEN + 1..=u8::MAX {
            assert!(Ipv4Cidr::try_new(Ipv4Addr::new(0, 0, 0, 0), network).is_err());
        }
    }

    #[test]
    fn test_ipv4_cidr_contains() {
        let cidr = Ipv4Cidr::try_new(Ipv4Addr::new(123, 45, 67, 8), 28).unwrap();
        for i in 0..1 << 4 {
            let addr = Ipv4Addr::new(123, 45, 67, i);
            assert!(cidr.contains(addr), "should contain {:?}", addr);
        }
        for i in 1 << 4..=u8::MAX {
            let addr = Ipv4Addr::new(123, 45, 67, i);
            assert!(!cidr.contains(addr), "should not contain {:?}", addr);
        }
    }

    #[test]
    fn test_ipv6_cidr_new() {
        for network in 0..=Ipv6Cidr::MAX_NET_LEN {
            assert!(Ipv6Cidr::try_new(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0), network).is_ok());
        }
        for network in Ipv6Cidr::MAX_NET_LEN + 1..=u8::MAX {
            assert!(Ipv6Cidr::try_new(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0), network).is_err());
        }
    }

    #[test]
    fn test_ipv6_cidr_contains() {
        let cidr = Ipv6Cidr::try_new(Ipv6Addr::new(123, 45, 67, 89, 10, 11, 12, 13), 120).unwrap();
        for i in 0..1 << 8 {
            let addr = Ipv6Addr::new(123, 45, 67, 89, 10, 11, 12, i);
            assert!(cidr.contains(addr), "should contain {:?}", addr);
        }
        #[cfg(not(miri))]
        let excluded_range = 1 << 8..=u16::MAX;
        #[cfg(miri)]
        let excluded_range = (1 << 8..=u16::MAX).step_by(1 << 8);
        for i in excluded_range {
            let addr = Ipv6Addr::new(123, 45, 67, 89, 10, 11, 12, i);
            assert!(!cidr.contains(addr), "should not contain {:?}", addr);
        }
    }
}
