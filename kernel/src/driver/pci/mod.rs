use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use bitflags::bitflags;
use conquer_once::spin::OnceCell;
use derive_more::Display;

pub use classes::*;
pub use device::*;
pub use header::*;

mod classes;
mod device;
mod header;
mod raw;

static DEVICES: OnceCell<Devices> = OnceCell::uninit();

pub fn devices() -> impl Iterator<Item = Weak<PciDevice>> {
    DEVICES
        .get_or_init(|| {
            let mut devices = Vec::new();
            for bus in 0..=255 {
                unsafe { raw::iterate_bus(bus, &mut devices) };
            }
            Devices {
                devices: devices.into_iter().map(Arc::new).collect::<Vec<_>>(),
            }
        })
        .iter()
}

pub struct Devices {
    devices: Vec<Arc<PciDevice>>,
}

impl Devices {
    pub fn iter(&self) -> DevicesIter {
        DevicesIter {
            devices: self,
            index: 0,
        }
    }
}

pub struct DevicesIter<'a> {
    devices: &'a Devices,
    index: usize,
}

impl<'a> Iterator for DevicesIter<'a> {
    type Item = Weak<PciDevice>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.devices.devices.len() {
            return None;
        }
        let item = &self.devices.devices[self.index];
        self.index += 1;
        Some(Arc::downgrade(item))
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Display)]
pub enum PciError {
    #[display("unknown header type {_0:#x?}")]
    UnknownHeaderType(u8),
    #[display("unknown pci device class {_0:#x?}")]
    UnknownPciDeviceClass(u16),
    #[display("unknown interrupt pin {_0}")]
    UnknownInterruptPin(u8),
    #[display("unknown display sub class {_0:#x?}")]
    UnknownDisplaySubClass(u8),
    #[display("unknown serial bus sub class {_0:#x?}")]
    UnknownSerialBusSubClass(u8),
    #[display("unknown mass storage sub class {_0:#x?}")]
    UnknownMassStorageSubClass(u8),
    #[display("unknown network sub class {_0:#x?}")]
    UnknownNetworkSubClass(u8),
    #[display("unknown bridge sub class {_0:#x?}")]
    UnknownBridgeSubClass(u8),

    #[display("not a standard header, but a {_0:?}")]
    NotStandardHeader(PciHeaderType),
    #[display("not a pci2pci bridge, but a {_0:?}")]
    NotPCI2PCIBridge(PciHeaderType),
}

impl core::error::Error for PciError {}

bitflags! {
    pub struct Status: u16 {
        const DETECTED_PARITY_ERROR = 1 << 15;
        const SIGNALED_SYSTEM_ERROR = 1 << 14;
        const RECEIVED_MASTER_ABORT = 1 << 13;
        const RECEIVED_TARGET_ABORT = 1 << 12;
        const SIGNALED_TARGET_ABORT = 1 << 11;
        const DEVSEL_TIMING = 1 << 10 | 1 << 9;
        const MASTER_DATA_PARITY_ERROR = 1 << 8 ;
        const FAST_BACK_TO_BACK_CAPABLE = 1 << 7;
        const MHZ66_CAPABLE = 1 << 5;
        const CAPABILITIES_LIST = 1 << 4;
        const INTERRUPT = 1 << 3;
    }
}

bitflags! {
    pub struct BIST: u8 {
        const BIST_CAPABLE = 1 << 7;
        const START_BIST = 1 << 6;
        const COMPLETION_CODE = (1 << 4) - 1;
    }
}
