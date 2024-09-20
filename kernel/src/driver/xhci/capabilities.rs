use core::fmt::{Debug, Formatter};
use volatile::access::{NoAccess, ReadOnly};
use volatile::VolatileFieldAccess;

/// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=380)
#[repr(C)]
#[derive(Debug, Copy, Clone, VolatileFieldAccess)]
pub struct Capabilities {
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=381)
    #[access(ReadOnly)]
    caplength: u8,
    #[access(NoAccess)]
    rsvd: u8,
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=381)
    #[access(ReadOnly)]
    hciversion: u16,
    /// [`HcsParams1`]
    #[access(ReadOnly)]
    hcsparams1: HcsParams1,
    /// [`HcsParams2`]
    #[access(ReadOnly)]
    hcsparams2: HcsParams2,
    /// [`HcsParams3`]
    #[access(ReadOnly)]
    hcsparams3: HcsParams3,
    /// [`HccParams1`]
    #[access(ReadOnly)]
    hccparams1: HccParams1,
    /// [`DbOff`]
    #[access(ReadOnly)]
    dboff: DbOff,
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=388)
    #[access(ReadOnly)]
    rtsoff: u32,
    /// [`HccParams2`]
    #[access(ReadOnly)]
    hccparms2: HccParams2,
    /// [`VtiosOff`]
    #[access(ReadOnly)]
    vtiosoff: VtiosOff,
}

/// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=382)
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct HcsParams1(u32);

impl HcsParams1 {
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=383)
    pub fn max_ports(&self) -> u8 {
        (self.0 >> 24) as u8
    }

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=382)
    pub fn max_device_slots(&self) -> u8 {
        self.0 as u8
    }

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=382)
    pub fn max_interrupters(&self) -> u16 {
        ((self.0 >> 8) & ((1 << 9) - 1)) as u16
    }
}

impl Debug for HcsParams1 {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("HcsParams1")
            .field("max_ports", &self.max_ports())
            .field("max_device_slots", &self.max_device_slots())
            .field("max_interrupters", &self.max_interrupters())
            .finish()
    }
}

/// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=383)
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct HcsParams2(u32);

impl HcsParams2 {
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=383)
    pub fn ist(&self) -> u8 {
        (self.0 & 0x111) as u8
    }

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=383)
    pub fn erst_max(&self) -> u16 {
        ((self.0 >> 3) & ((1 << 4) - 1)) as u16
    }

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=384)
    pub fn max_scratchpad_bufs_hi(&self) -> u8 {
        ((self.0 >> 20) & ((1 << 5) - 1)) as u8
    }

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=384)
    pub fn max_scratchpad_bufs_lo(&self) -> u8 {
        (self.0 >> 27) as u8
    }

    /// See [`max_scratchpad_bufs_hi`](HcsParams2::max_scratchpad_bufs_hi) and
    /// [`max_scratchpad_bufs_lo`](HcsParams2::max_scratchpad_bufs_lo).
    ///
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=384)
    pub fn max_scratchpad_bufs(&self) -> u16 {
        ((self.max_scratchpad_bufs_hi() as u16) << 8) | self.max_scratchpad_bufs_lo() as u16
    }

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=384)
    pub fn scratchpad_restore(&self) -> bool {
        (self.0 & (1 << 26)) != 0
    }
}

impl Debug for HcsParams2 {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("HcsParams2")
            .field("ist", &self.ist())
            .field("erst_max", &self.erst_max())
            .field("max_scratchpad_bufs_hi", &self.max_scratchpad_bufs_hi())
            .field("max_scratchpad_bufs_lo", &self.max_scratchpad_bufs_lo())
            .field("max_scratchpad_bufs", &self.max_scratchpad_bufs())
            .field("scratchpad_restore", &self.scratchpad_restore())
            .finish()
    }
}

/// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=384)
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct HcsParams3(u32);

impl HcsParams3 {
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=384)
    pub fn u1_device_exit_latency(&self) -> u8 {
        self.0 as u8
    }

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=385)
    pub fn u2_device_exit_latency(&self) -> u16 {
        (self.0 >> 16) as u16
    }
}

impl Debug for HcsParams3 {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("HcsParams3")
            .field("u1_device_exit_latency", &self.u1_device_exit_latency())
            .field("u2_device_exit_latency", &self.u2_device_exit_latency())
            .finish()
    }
}

/// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=385)
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct HccParams1(u32);

impl HccParams1 {
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=386)
    pub fn ac64(&self) -> bool {
        (self.0 & (1 << 0)) != 0
    }

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=386)
    pub fn bnc(&self) -> bool {
        (self.0 & (1 << 1)) != 0
    }

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=386)
    pub fn csz(&self) -> bool {
        (self.0 & (1 << 2)) != 0
    }

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=386)
    pub fn ppc(&self) -> bool {
        (self.0 & (1 << 3)) != 0
    }

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=386)
    pub fn pind(&self) -> bool {
        (self.0 & (1 << 4)) != 0
    }

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=386)
    pub fn lhrc(&self) -> bool {
        (self.0 & (1 << 5)) != 0
    }

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=387)
    pub fn ltc(&self) -> bool {
        (self.0 & (1 << 6)) != 0
    }

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=387)
    pub fn nss(&self) -> bool {
        (self.0 & (1 << 7)) != 0
    }

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=387)
    pub fn pae(&self) -> bool {
        (self.0 & (1 << 8)) != 0
    }

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=387)
    pub fn spc(&self) -> bool {
        (self.0 & (1 << 9)) != 0
    }

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=387)
    pub fn sec(&self) -> bool {
        (self.0 & (1 << 10)) != 0
    }

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=387)
    pub fn cfc(&self) -> bool {
        (self.0 & (1 << 11)) != 0
    }

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=387)
    pub fn max_psa_size(&self) -> u8 {
        ((self.0 >> 12) & ((1 << 4) - 1)) as u8
    }

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=387)
    pub fn xecp(&self) -> u16 {
        (self.0 >> 16) as u16
    }
}

impl Debug for HccParams1 {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("HccParams1")
            .field("ac64", &self.ac64())
            .field("bnc", &self.bnc())
            .field("csz", &self.csz())
            .field("ppc", &self.ppc())
            .field("pind", &self.pind())
            .field("lhrc", &self.lhrc())
            .field("ltc", &self.ltc())
            .field("nss", &self.nss())
            .field("pae", &self.pae())
            .field("spc", &self.spc())
            .field("sec", &self.sec())
            .field("cfc", &self.cfc())
            .field("max_psa_size", &self.max_psa_size())
            .field("xecp", &self.xecp())
            .finish()
    }
}

/// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=387)
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct DbOff(u32);

impl DbOff {
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=388)
    pub fn offset(&self) -> u32 {
        self.0 >> 2
    }
}

impl Debug for DbOff {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DbOff")
            .field("offset", &self.offset())
            .finish()
    }
}

/// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=389)
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct HccParams2(u32);

impl HccParams2 {
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=389)
    pub fn u3c(&self) -> bool {
        (self.0 & (1 << 0)) != 0
    }

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=389)
    pub fn cmc(&self) -> bool {
        (self.0 & (1 << 1)) != 0
    }

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=389)
    pub fn fsc(&self) -> bool {
        (self.0 & (1 << 2)) != 0
    }

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=390)
    pub fn ctc(&self) -> bool {
        (self.0 & (1 << 3)) != 0
    }

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=390)
    pub fn lec(&self) -> bool {
        (self.0 & (1 << 4)) != 0
    }

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=390)
    pub fn cic(&self) -> bool {
        (self.0 & (1 << 5)) != 0
    }

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=390)
    pub fn etc(&self) -> bool {
        (self.0 & (1 << 6)) != 0
    }

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=390)
    pub fn etc_tsc(&self) -> bool {
        (self.0 & (1 << 7)) != 0
    }

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=390)
    pub fn gsc(&self) -> bool {
        (self.0 & (1 << 8)) != 0
    }

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=390)
    pub fn vtc(&self) -> bool {
        (self.0 & (1 << 9)) != 0
    }
}

impl Debug for HccParams2 {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("HccParams2")
            .field("u3c", &self.u3c())
            .field("cmc", &self.cmc())
            .field("fsc", &self.fsc())
            .field("ctc", &self.ctc())
            .field("lec", &self.lec())
            .field("cic", &self.cic())
            .field("etc", &self.etc())
            .field("etc_tsc", &self.etc_tsc())
            .field("gsc", &self.gsc())
            .field("vtc", &self.vtc())
            .finish()
    }
}

/// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=390)
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct VtiosOff(u32);

impl VtiosOff {
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=391)
    pub fn offset(&self) -> u32 {
        self.0 >> 12
    }
}

impl Debug for VtiosOff {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("VtiosOff")
            .field("offset", &self.offset())
            .finish()
    }
}
