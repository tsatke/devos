use crate::driver::usb::protocol_support::ProtocolSupport;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Speed {
    protocols: ProtocolSupport,
    plt: u8,
    pfd: bool,
    psie: Psie,
    psim: u16,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
enum Psie {
    B,
    KB,
    MB,
    GB,
}

impl Speed {
    pub const FULL_SPEED: Self = Self::new(ProtocolSupport::USB2_ONLY, 0, false, 2, 12).unwrap();

    pub const LOW_SPEED: Self = Self::new(ProtocolSupport::USB2_ONLY, 0, false, 1, 1500).unwrap();

    pub const HIGH_SPEED: Self = Self::new(ProtocolSupport::USB2_ONLY, 0, false, 2, 480).unwrap();

    pub const SUPER_SPEED_GEN1_X1: Self =
        Self::new(ProtocolSupport::USB3_ANY, 0, true, 3, 5).unwrap();

    pub const SUPER_SPEED_GEN2_X1: Self =
        Self::new(ProtocolSupport::USB3_1_ONLY, 0, true, 3, 10).unwrap();

    // although the spec says that this is 5Gb/s, the actual bit rate is 10Gb/s
    pub const SUPER_SPEED_GEN1_X2: Self =
        Self::new(ProtocolSupport::USB3_2_ONLY, 0, true, 3, 10).unwrap();

    pub const SUPER_SPEED_GEN2_X2: Self =
        Self::new(ProtocolSupport::USB3_2_ONLY, 0, true, 3, 20).unwrap();

    pub const fn new(
        protocols: ProtocolSupport,
        plt: u8,
        pfd: bool,
        psie: u8,
        psim: u16,
    ) -> Option<Self> {
        Some(Self {
            protocols,
            plt,
            pfd,
            psie: match psie {
                0 => Psie::B,
                1 => Psie::KB,
                2 => Psie::MB,
                3 => Psie::GB,
                _ => return None,
            },
            psim,
        })
    }

    pub fn bit_rate(&self) -> usize {
        let magnitude = match self.psie {
            Psie::B => 1,
            Psie::KB => 1000,
            Psie::MB => 1_000_000,
            Psie::GB => 1_000_000_000,
        };
        usize::from(self.psim) * magnitude
    }
}

#[cfg(feature = "kernel_test")]
mod tests {
    use crate::driver::usb::speed::Speed;
    use kernel_test_framework::kernel_test;

    #[kernel_test]
    fn test_usb_speed_bit_rate() {
        assert_eq!(Speed::FULL_SPEED.bit_rate(), 12_000_000);
        assert_eq!(Speed::LOW_SPEED.bit_rate(), 1_500_000);
        assert_eq!(Speed::HIGH_SPEED.bit_rate(), 480_000_000);
        assert_eq!(Speed::SUPER_SPEED_GEN1_X1.bit_rate(), 5_000_000_000);
        assert_eq!(Speed::SUPER_SPEED_GEN2_X1.bit_rate(), 10_000_000_000);
        assert_eq!(Speed::SUPER_SPEED_GEN1_X2.bit_rate(), 10_000_000_000);
        assert_eq!(Speed::SUPER_SPEED_GEN2_X2.bit_rate(), 20_000_000_000);
    }
}
