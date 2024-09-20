use bitfield::bitfield;
use core::fmt::Debug;
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

bitfield! {
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=382)
    #[repr(transparent)]
    #[derive(Copy, Clone)]
    pub struct HcsParams1(u32);
    impl Debug;

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=382)
    pub u8, max_device_slots, _: 7, 0;
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=382)
    pub u16, max_interrupters, _: 18, 8;
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=383)
    pub u8, max_ports, _: 31, 24;
}

bitfield! {
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=383)
    #[repr(transparent)]
    #[derive(Copy, Clone)]
    pub struct HcsParams2(u32);
    impl Debug;

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=383)
    pub u8, ist, _: 3, 0;

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=383)
    pub u8, erst_max, _: 7, 4;

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=384)
    pub u8, max_scratchpad_bufs_hi, _: 25, 21;

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=384)
    pub u8, max_scratchpad_bufs_lo, _: 31, 27;

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=384)
    pub scratchpad_restore, _: 26;
}

impl HcsParams2 {
    /// See [`max_scratchpad_bufs_hi`](HcsParams2::max_scratchpad_bufs_hi) and
    /// [`max_scratchpad_bufs_lo`](HcsParams2::max_scratchpad_bufs_lo).
    ///
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=384)
    pub fn max_scratchpad_bufs(&self) -> u16 {
        ((self.max_scratchpad_bufs_hi() as u16) << 8) | self.max_scratchpad_bufs_lo() as u16
    }
}

bitfield! {
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=384)
    #[repr(transparent)]
    #[derive(Copy, Clone)]
    pub struct HcsParams3(u32);
    impl Debug;

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=384)
    pub u8, u1_device_exit_latency, _: 7, 0;
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=385)
    pub u16, u2_device_exit_latency, _: 31, 16;
}

bitfield! {
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=385)
    #[repr(transparent)]
    #[derive(Copy, Clone)]
    pub struct HccParams1(u32);
    impl Debug;

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=386)
    pub ac64, _: 0;

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=386)
    pub bnc, _: 1;

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=386)
    pub csz, _: 2;

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=386)
    pub ppc, _: 3;

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=386)
    pub pind, _: 4;

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=386)
    pub lhrc, _: 5;

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=387)
    pub ltc, _: 6;

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=387)
    pub nss, _: 7;

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=387)
    pub pae, _: 8;

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=387)
    pub spc, _: 9;

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=387)
    pub sec, _: 10;

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=387)
    pub cfc, _: 11;

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=387)
    pub u8, max_psa_size, _: 15, 12;

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=387)
    pub u16, xecp, _: 31, 16;
}

bitfield! {
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=387)
    #[repr(transparent)]
    #[derive(Copy, Clone)]
    pub struct DbOff(u32);
    impl Debug;

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=388)
    pub offset, _: 31, 2;
}

bitfield! {
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=389)
    #[repr(transparent)]
    #[derive(Copy, Clone)]
    pub struct HccParams2(u32);
    impl Debug;

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=389)
    pub u3c, _: 0;
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=389)
    pub cmc, _: 1;
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=389)
    pub fsc, _: 2;
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=390)
    pub ctc, _: 3;
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=390)
    pub lec, _: 4;
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=390)
    pub cic, _: 5;
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=390)
    pub etc, _: 6;
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=390)
    pub etc_tsc, _: 7;
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=390)
    pub gsc, _: 8;
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=390)
    pub vtc, _: 9;
}

bitfield! {
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=390)
    #[repr(transparent)]
    #[derive(Copy, Clone)]
    pub struct VtiosOff(u32);
    impl Debug;

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=391)
    pub offset, _: 31, 12;
}