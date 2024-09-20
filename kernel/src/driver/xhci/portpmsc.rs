pub trait PortPmsc {}

/// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=415)
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct PortPmscUsb3(u32);

impl PortPmsc for PortPmscUsb3 {}

impl PortPmscUsb3 {
    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=415)
    pub fn u1_timeout(&self) -> u8 {
        self.0 as u8
    }

    /// [`Self::u1_timeout`]
    pub fn set_u1_timout(&mut self, value: u8) {
        self.0 = (self.0 & !0xFF) | value as u32;
    }

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=416)
    pub fn u2_timeout(&self) -> u8 {
        (self.0 >> 8) as u8
    }

    /// [`Self::u2_timeout`]
    pub fn set_u2_timout(&mut self, value: u8) {
        self.0 = (self.0 & !(0xFF << 8)) | ((value as u32) << 8);
    }

    /// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=416)
    pub fn fla(&self) -> bool {
        self.0 & (1 << 16) != 0
    }

    pub fn set_fla(&mut self, value: bool) {
        self.0 = (self.0 & !(1 << 16)) | ((if value { 1_u32 } else { 0_u32 }) << 16);
    }
}

/// [USB xHCI spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/extensible-host-controler-interface-usb-xhci.pdf#page=416)
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct PortPmscUsb2(u32);

impl PortPmsc for PortPmscUsb2 {}
