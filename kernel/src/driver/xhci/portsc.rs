use core::fmt;
use core::fmt::{Debug, Formatter};

/// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=405)
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct PortSc(u32);

impl PortSc {
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=405)
    pub const CCS: PortSc = PortSc(1 << 0);

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=407)
    pub const PED: PortSc = PortSc(1 << 1);

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=407)
    pub const OCA: PortSc = PortSc(1 << 3);

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=407)
    pub const PR: PortSc = PortSc(1 << 4);

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=409)
    pub const PP: PortSc = PortSc(1 << 9);

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=410)
    pub const LWS: PortSc = PortSc(1 << 16);

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=410)
    pub const CSC: PortSc = PortSc(1 << 17);

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=410)
    pub const PEC: PortSc = PortSc(1 << 18);

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=411)
    pub const WRC: PortSc = PortSc(1 << 19);

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=411)
    pub const OCC: PortSc = PortSc(1 << 20);

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=411)
    pub const PRC: PortSc = PortSc(1 << 21);

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=412)
    pub const PLC: PortSc = PortSc(1 << 22);

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=412)
    pub const CEC: PortSc = PortSc(1 << 23);

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=406)
    pub const CAS: PortSc = PortSc(1 << 24);

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=406)
    pub const WCE: PortSc = PortSc(1 << 25);

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=406)
    pub const WDE: PortSc = PortSc(1 << 26);

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=406)
    pub const WOE: PortSc = PortSc(1 << 27);

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=406)
    pub const DR: PortSc = PortSc(1 << 30);

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=406)
    pub const WPR: PortSc = PortSc(1 << 31);

    const PLS_SHIFT: usize = 5;
    const PLS_MASK: u32 = ((1 << 4) - 1) << Self::PLS_SHIFT;
    const PORT_SPEED_SHIFT: usize = 10;
    const PORT_SPEED_MASK: u32 = ((1 << 4) - 1) << Self::PORT_SPEED_SHIFT;
    const PIC_SHIFT: usize = 14;
    const PIC_MASK: u32 = ((1 << 2) - 1) << Self::PIC_SHIFT;

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=408)
    pub fn pls(&self) -> u8 {
        ((self.0 & Self::PLS_MASK) >> Self::PLS_SHIFT) as u8
    }

    /// [`Self::pls`]
    pub fn set_pls(&mut self, value: u8) {
        (*self).0 &= !Self::PLS_MASK;
        (*self).0 |= (value as u32) << Self::PLS_SHIFT;
    }

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=410)
    pub fn port_speed(&self) -> u8 {
        ((self.0 & Self::PORT_SPEED_MASK) >> Self::PORT_SPEED_SHIFT) as u8
    }

    /// [`Self::port_speed`]
    pub fn set_port_speed(&mut self, value: u8) {
        (*self).0 &= !Self::PORT_SPEED_MASK;
        (*self).0 |= (value as u32) << Self::PORT_SPEED_SHIFT;
    }

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=410)
    pub fn pic(&self) -> u8 {
        ((self.0 & Self::PIC_MASK) >> Self::PIC_SHIFT) as u8
    }

    /// [`Self::pic`]
    pub fn set_pic(&mut self, value: u8) {
        (*self).0 &= !Self::PIC_MASK;
        (*self).0 |= (value as u32) << Self::PIC_SHIFT;
    }

    pub fn contains(&self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    pub fn set(&mut self, other: Self, value: bool) {
        if value {
            (*self).0 |= other.0;
        } else {
            (*self).0 = self.0 & !other.0;
        }
    }
}

impl Debug for PortSc {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("PortSc")
            .field("ccs", &self.contains(Self::CCS))
            .field("ped", &self.contains(Self::PED))
            .field("oca", &self.contains(Self::OCA))
            .field("pr", &self.contains(Self::PR))
            .field("pls", &self.pls())
            .field("pp", &self.contains(Self::PP))
            .field("port_speed", &self.port_speed())
            .field("pic", &self.pic())
            .field("lws", &self.contains(Self::LWS))
            .field("csc", &self.contains(Self::CSC))
            .field("pec", &self.contains(Self::PEC))
            .field("wrc", &self.contains(Self::WRC))
            .field("occ", &self.contains(Self::OCC))
            .field("prc", &self.contains(Self::PRC))
            .field("plc", &self.contains(Self::PLC))
            .field("cec", &self.contains(Self::CEC))
            .field("cas", &self.contains(Self::CAS))
            .field("wce", &self.contains(Self::WCE))
            .field("wde", &self.contains(Self::WDE))
            .field("woe", &self.contains(Self::WOE))
            .field("dr", &self.contains(Self::DR))
            .field("wpr", &self.contains(Self::WPR))
            .finish()
    }
}