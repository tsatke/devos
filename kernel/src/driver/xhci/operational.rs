use bitflags::bitflags;
use core::fmt;
use core::fmt::{Debug, Formatter};
use volatile::access::{ReadOnly, ReadWrite};
use volatile::VolatileFieldAccess;

/// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=391)
#[repr(C)]
#[derive(Debug, Copy, Clone, VolatileFieldAccess)]
pub struct Operational {
    /// [`UsbCmd`]
    #[access(ReadWrite)]
    usbcmd: UsbCmd,
    /// [`UsbSts`]
    #[access(ReadWrite)]
    usbsts: UsbSts,
    /// [`Pagesize`]
    #[access(ReadOnly)]
    pagesize: Pagesize,
    /// [`DnCtrl`]
    #[access(ReadWrite)]
    dnctrl: DnCtrl,
    /// [`Crcr`]
    #[access(ReadWrite)]
    crcr: Crcr,
    /// [`Dcbaap`]
    #[access(ReadWrite)]
    dcbaap: Dcbaap,
    /// [`Config`]
    #[access(ReadWrite)]
    config: Config,
}

bitflags! {
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=393)
    #[repr(transparent)]
    #[derive(Debug, Copy, Clone)]
    pub struct UsbCmd: u32 {
        /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=393)
        const RS = 1 << 0;

        /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=394)
        const HCRST = 1 << 1;

        /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=394)
        const INTE = 1 << 2;

        /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=394)
        const HSEE = 1 << 3;

        /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=395)
        const LHCRST = 1 << 7;

        /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=395)
        const CSS = 1 << 8;

        /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=395)
        const CRS = 1 << 9;

        /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=395)
        const EWE = 1 << 10;

        /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=395)
        const EU3S = 1 << 11;

        /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=396)
        const CME = 1 << 13;

        /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=396)
        const ETE = 1 << 14;

        /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=396)
        const TSC_EN = 1 << 15;

        /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=396)
        const VTIOE = 1 << 16;
    }
}

bitflags! {
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=397)
    #[repr(transparent)]
    #[derive(Debug, Copy, Clone)]
    pub struct UsbSts: u32 {
        /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=398)
        const HCH = 1 << 0;

        /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=398)
        const HSE = 1 << 2;

        /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=398)
        const EINT = 1 << 3;

        /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=398)
        const PCD = 1 << 4;

        /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=399)
        const SSS = 1 << 8;

        /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=399)
        const RSS = 1 << 9;

        /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=399)
        const SRE = 1 << 10;

        /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=399)
        const CNR = 1 << 11;

        /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=399)
        const HCE = 1 << 12;
    }
}

/// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=399)
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct Pagesize(u32);

impl Pagesize {
    /// [`Pagesize`]
    pub fn size_raw(&self) -> u32 {
        self.0 & ((1 << 16) - 1)
    }

    /// [`Pagesize`]
    pub fn size(&self) -> u32 {
        1 << (self.size_raw().trailing_zeros() + 12)
    }
}

impl Debug for Pagesize {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Pagesize")
            .field("size_raw", &self.size_raw())
            .field("size", &self.size())
            .finish()
    }
}

bitflags! {
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=400)
    #[repr(transparent)]
    #[derive(Debug, Copy, Clone)]
    pub struct DnCtrl: u32 {
        /// [`DnCtrl`]
        const N0 = 1 << 0;
        /// [`DnCtrl`]
        const N1 = 1 << 1;
        /// [`DnCtrl`]
        const N2 = 1 << 2;
        /// [`DnCtrl`]
        const N3 = 1 << 3;
        /// [`DnCtrl`]
        const N4 = 1 << 4;
        /// [`DnCtrl`]
        const N5 = 1 << 5;
        /// [`DnCtrl`]
        const N6 = 1 << 6;
        /// [`DnCtrl`]
        const N7 = 1 << 7;
        /// [`DnCtrl`]
        const N8 = 1 << 8;
        /// [`DnCtrl`]
        const N9 = 1 << 9;
        /// [`DnCtrl`]
        const N10 = 1 << 10;
        /// [`DnCtrl`]
        const N11 = 1 << 11;
        /// [`DnCtrl`]
        const N12 = 1 << 12;
        /// [`DnCtrl`]
        const N13 = 1 << 13;
        /// [`DnCtrl`]
        const N14 = 1 << 14;
        /// [`DnCtrl`]
        const N15 = 1 << 15;
    }
}

/// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=401)
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct Crcr(u64);

impl Crcr {
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=402)
    pub const RCS: Crcr = Crcr(1 << 0);

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=402)
    pub const CS: Crcr = Crcr(1 << 1);

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=402)
    pub const CA: Crcr = Crcr(1 << 2);

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=402)
    pub const CRR: Crcr = Crcr(1 << 3);

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=403)
    pub fn command_ring_pointer(&self) -> u64 {
        0
    }

    /// [`Self::command_ring_pointer`]
    pub fn set_command_ring_pointer(&mut self, value: u64) {
        (*self).0 &= (1 << 6) - 1;
        (*self).0 |= value << 6;
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

impl Debug for Crcr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Crcr")
            .field("RCS", &self.contains(Crcr::RCS))
            .field("CS", &self.contains(Crcr::CS))
            .field("CA", &self.contains(Crcr::CA))
            .field("CRR", &self.contains(Crcr::CRR))
            .field("command_ring_pointer", &self.command_ring_pointer())
            .finish()
    }
}

/// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=403)
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct Dcbaap(u64);

impl Dcbaap {
    /// [`Dcbaap`]
    pub fn pointer(&self) -> u64 {
        self.0 >> 6
    }
}

impl Debug for Dcbaap {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Dcbaap")
            .field("pointer", &(self.pointer() as *const ()))
            .finish()
    }
}

/// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=404)
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct Config(u32);

impl Config {
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=404)
    pub const U3E: Config = Config(1 << 8);

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=405)
    pub const CIE: Config = Config(1 << 9);

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=404)
    pub fn max_device_slots_enabled(&self) -> u8 {
        self.0 as u8
    }

    /// [`Self::max_device_slots_enabled`]
    pub fn set_max_device_slots_enabled(&mut self, value: u8) {
        (*self).0 &= !((1 << 8) - 1);
        (*self).0 |= value as u32;
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

impl Debug for Config {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Config")
            .field("U3E", &self.contains(Config::U3E))
            .field("CIE", &self.contains(Config::CIE))
            .field("max_device_slots_enabled", &self.max_device_slots_enabled())
            .finish()
    }
}
