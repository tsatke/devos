use alloc::boxed::Box;
use core::alloc::AllocError;
use core::error::Error;
use core::fmt::{Debug, Display, Formatter};

use crate::driver::ide::controller::IdeController;
use crate::driver::pci::{PciDriverDescriptor, PCI_DRIVERS};
use bitflags::bitflags;
use conquer_once::spin::OnceCell;
pub use device::*;
use foundation::falloc::vec::FVec;
use linkme::distributed_slice;
use spin::Mutex;

mod channel;
mod command;
mod controller;
mod device;
mod drive;

#[distributed_slice(PCI_DRIVERS)]
static IDE_CONTROLLER_DRIVER: PciDriverDescriptor = PciDriverDescriptor {
    name: "IDEController",
    probe: IdeController::probe,
    init: IdeController::init,
};

static IDE_DEVICES: OnceCell<Mutex<FVec<IdeBlockDevice>>> = OnceCell::uninit();

fn register_ide_block_device(device: IdeBlockDevice) -> Result<(), Box<dyn Error>> {
    match devices().lock().try_push(device) {
        Ok(_) => Ok(()),
        Err(_e) => Err(Box::new(AllocError)),
    }
}

pub fn devices() -> &'static Mutex<FVec<IdeBlockDevice>> {
    IDE_DEVICES.get_or_init(Mutex::default)
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

impl Error for IdeError {}

fn is_bit_set(haystack: u64, needle: u8) -> bool {
    (haystack & (1 << needle)) > 0
}
