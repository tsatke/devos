use crate::driver::usb::Protocol;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ProtocolSupport {
    usb2: bool,
    usb3: bool,
    usb3_1: bool,
    usb3_2: bool,
}

impl ProtocolSupport {
    pub const USB2_ONLY: Self = Self::new(true, false, false, false);
    pub const USB3_ONLY: Self = Self::new(false, true, false, false);
    pub const USB3_ANY: Self = Self::new(false, true, true, true);
    pub const USB3_1_ONLY: Self = Self::new(false, false, true, false);
    pub const USB3_2_ONLY: Self = Self::new(false, false, false, true);

    pub const fn new(usb2: bool, usb3: bool, usb3_1: bool, usb3_2: bool) -> Self {
        Self {
            usb2,
            usb3,
            usb3_1,
            usb3_2,
        }
    }

    pub const fn supports(&self, protocol: Protocol) -> bool {
        match protocol {
            Protocol::USB2 => self.usb2,
            Protocol::USB3 => self.usb3,
            Protocol::USB3_1 => self.usb3_1,
            Protocol::USB3_2 => self.usb3_2,
        }
    }
}

#[cfg(feature = "kernel_test")]
mod tests {
    use super::*;
    use kernel_test_framework::kernel_test;

    #[kernel_test]
    fn test_protocol_support() {
        let support = ProtocolSupport::USB2_ONLY;
        assert!(support.supports(Protocol::USB2));
        assert!(!support.supports(Protocol::USB3));
        assert!(!support.supports(Protocol::USB3_1));
        assert!(!support.supports(Protocol::USB3_2));

        let support = ProtocolSupport::USB3_ANY;
        assert!(!support.supports(Protocol::USB2));
        assert!(support.supports(Protocol::USB3));
        assert!(support.supports(Protocol::USB3_1));
        assert!(support.supports(Protocol::USB3_2));

        let support = ProtocolSupport::USB3_ONLY;
        assert!(!support.supports(Protocol::USB2));
        assert!(support.supports(Protocol::USB3));
        assert!(!support.supports(Protocol::USB3_1));
        assert!(!support.supports(Protocol::USB3_2));

        let support = ProtocolSupport::USB3_1_ONLY;
        assert!(!support.supports(Protocol::USB2));
        assert!(!support.supports(Protocol::USB3));
        assert!(support.supports(Protocol::USB3_1));
        assert!(!support.supports(Protocol::USB3_2));

        let support = ProtocolSupport::USB3_2_ONLY;
        assert!(!support.supports(Protocol::USB2));
        assert!(!support.supports(Protocol::USB3));
        assert!(!support.supports(Protocol::USB3_1));
        assert!(support.supports(Protocol::USB3_2));
    }
}
