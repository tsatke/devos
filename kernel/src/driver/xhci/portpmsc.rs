use bitfield::bitfield;

pub trait PortPmsc {}

bitfield! {
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=415)
    #[repr(transparent)]
    #[derive(Copy, Clone)]
    pub struct PortPmscUsb3(u32);
    impl Debug;

    pub u8, u1_timeout, set_u1_timout: 7, 0;
    pub u8, u2_timeout, set_u2_timout: 15, 8;
    pub fla, set_fla: 16;
}

bitfield! {
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=416)
    #[repr(transparent)]
    #[derive(Copy, Clone)]
    pub struct PortPmscUsb2(u32);
    impl Debug;

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=417)
    pub u8, l1s, _: 2, 0;
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=417)
    pub rwe, set_rwe: 3;
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=417)
    pub u8, besl, set_besl: 7, 4;
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=417)
    pub u8, l1_device_slot, set_up_l1_device_slot: 15, 8;
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=417)
    pub hle, set_hle: 16;
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=418)
    pub u8, test_mode, set_test_mode: 31, 28;
}

impl PortPmsc for PortPmscUsb2 {}
