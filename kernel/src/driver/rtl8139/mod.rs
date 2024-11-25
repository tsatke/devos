use crate::driver::pci::{PciDevice, PciDriverDescriptor, PCI_DRIVERS};
use crate::net;
use alloc::boxed::Box;
use alloc::sync::{Arc, Weak};
use core::error::Error;
use foundation::net::MacAddr;
use linkme::distributed_slice;
use netstack::device::{Device, RawDataLinkFrame};
use thiserror::Error;

#[distributed_slice(PCI_DRIVERS)]
static RTL8239_DRIVER: PciDriverDescriptor = PciDriverDescriptor {
    name: "RTL8139",
    probe: Rtl8139::probe,
    init: Rtl8139::init,
};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Error)]
pub enum TryFromPciDeviceError {
    #[error("device is not a RTL8139")]
    NotRtl8139,
    #[error("device is not connected")]
    DeviceDisconnected,
}

impl TryFrom<Weak<PciDevice>> for Rtl8139 {
    type Error = TryFromPciDeviceError;

    fn try_from(device: Weak<PciDevice>) -> Result<Self, Self::Error> {
        let device = device
            .upgrade()
            .ok_or(TryFromPciDeviceError::DeviceDisconnected)?;

        if !Rtl8139::probe(&device) {
            return Err(TryFromPciDeviceError::NotRtl8139);
        }

        Ok(Self {
            _pci_device: Arc::downgrade(&device),
        })
    }
}

pub struct Rtl8139 {
    _pci_device: Weak<PciDevice>,
}

impl Rtl8139 {
    pub const VENDOR_ID: u16 = 0x10EC;
    pub const DEVICE_ID: u16 = 0x8139;

    pub fn probe(device: &PciDevice) -> bool {
        device.vendor() == Self::VENDOR_ID && device.device() == Self::DEVICE_ID
    }

    pub fn init(device: Weak<PciDevice>) -> Result<(), Box<dyn Error>> {
        let _rtl8139 = Self {
            _pci_device: device,
        };
        net::register_nic(Box::new(_rtl8139))?;
        Ok(())
    }
}

impl Device for Rtl8139 {
    fn mac_address(&self) -> MacAddr {
        todo!()
    }

    fn try_read_frame(&self) -> Option<RawDataLinkFrame> {
        todo!()
    }

    fn try_write_frame(&self, _frame: RawDataLinkFrame) -> Result<(), RawDataLinkFrame> {
        todo!()
    }
}
