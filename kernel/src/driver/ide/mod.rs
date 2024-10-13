use alloc::vec::Vec;
use core::fmt::{Debug, Display, Formatter};

use bitflags::bitflags;
use conquer_once::spin::OnceCell;

use crate::driver::ide::controller::IdeController;
use crate::driver::pci::PciStandardHeaderDevice;
use crate::driver::pci::{MassStorageSubClass, PciDeviceClass};
pub use device::*;

mod channel;
mod command;
mod controller;
mod device;
mod drive;

static IDE_DEVICES: OnceCell<Vec<IdeBlockDevice>> = OnceCell::uninit();

pub fn drives() -> impl Iterator<Item = &'static IdeBlockDevice> {
    IDE_DEVICES.get_or_init(collect_devices).iter()
}

fn collect_devices() -> Vec<IdeBlockDevice> {
    crate::driver::pci::devices()
        .filter(|dev| {
            matches!(
                dev.class(),
                PciDeviceClass::MassStorageController(MassStorageSubClass::IDEController)
            )
        })
        .map(|dev| PciStandardHeaderDevice::new(dev.clone()).unwrap())
        .map(IdeController::from)
        .flat_map(|ctrl| ctrl.drives)
        .filter(|drive| drive.exists())
        .map(IdeBlockDevice::from)
        .collect()
}

bitflags! {
    #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
    pub struct UDMAMode: u8 {
        const UDMA_1 = 1 << 0;
        const UDMA_2 = 1 << 1;
        const UDMA_3 = 1 << 2;
        const UDMA_4 = 1 << 3;
        const UDMA_5 = 1 << 4;
        const UDMA_6 = 1 << 5;
        const UDMA_7 = 1 << 6;
    }
}

bitflags! {
    pub struct Status: u8 {
        const ERROR = 1 << 0;
        const INDEX = 1 << 1;
        const CORRECTED_DATA = 1 << 2;
        const DATA_READY = 1 << 3; // DRQ
        const OVERLAPPED_MODE_SERVICE_REQUEST = 1 << 4;
        const DRIVE_FAULT_ERROR = 1 << 5;
        const READY = 1 << 6;
        const BUSY = 1 << 7;
    }
}

bitflags! {
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub struct IdeError: u8 {
        const ADDRESS_MARK_NOT_FOUND = 1 << 0;
        const TRACK_ZERO_NOT_FOUND = 1 << 1;
        const ABORTED_COMMAND = 1 << 2;
        const MEDIA_CHANGE_REQUEST = 1 << 3;
        const ID_NOT_FOUND = 1 << 4;
        const MEDIA_CHANGED = 1 << 5;
        const UNCORRECTABLE_DATA_ERROR = 1 << 6;
        const BAD_BLOCK_DETECTED = 1 << 7;
    }
}

impl Display for IdeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(self, f)
    }
}

impl core::error::Error for IdeError {}

fn is_bit_set(haystack: u64, needle: u8) -> bool {
    (haystack & (1 << needle)) > 0
}
