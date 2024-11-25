use crate::driver::pci::{PciDevice, PciDriverDescriptor, PCI_DRIVERS};
use alloc::boxed::Box;
use alloc::sync::Weak;
use core::error::Error;
use linkme::distributed_slice;

#[distributed_slice(PCI_DRIVERS)]
static RTL8239_DRIVER: PciDriverDescriptor = PciDriverDescriptor {
    name: "RTL8139",
    probe: Rtl8139::probe,
    init: Rtl8139::init,
};

pub struct Rtl8139 {
    pci_device: Weak<PciDevice>,
}

impl Rtl8139 {
    pub const VENDOR_ID: u16 = 0x10EC;
    pub const DEVICE_ID: u16 = 0x8139;

    pub fn probe(device: &PciDevice) -> bool {
        device.vendor() == Self::VENDOR_ID && device.device() == Self::DEVICE_ID
    }

    pub fn init(device: Weak<PciDevice>) -> Result<(), Box<dyn Error>> {
        let _rtl8139 = Self { pci_device: device };
        // TODO: register the device with the network stack
        Ok(())
    }
}
