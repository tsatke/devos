use alloc::boxed::Box;
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use bitflags::bitflags;
use conquer_once::spin::OnceCell;
use core::error::Error;
use linkme::distributed_slice;
use log::{error, info, trace, warn};
use spin::Mutex;

pub use device::*;

mod device;
mod raw;
mod register;

#[distributed_slice]
pub static PCI_DRIVERS: [PciDriverDescriptor];

pub fn init() {
    PCI_DRIVERS
        .iter()
        .map(|driver| driver.name)
        .for_each(|name| {
            trace!("have driver: {}", name);
        });

    devices().for_each(|dev| {
        for driver in PCI_DRIVERS.iter() {
            if (driver.probe)(&dev.lock()) {
                match (driver.init)(Arc::downgrade(dev)) {
                    Ok(_) => {
                        info!("loaded driver {} for {}", driver.name, dev.lock());
                        return;
                    }
                    Err(e) => {
                        error!(
                            "failed to load driver {} for {}: {}",
                            driver.name,
                            dev.lock(),
                            e
                        )
                    }
                }
            }
        }
        warn!("no driver found for {}", dev.lock());
    });
}

static DEVICES: OnceCell<Devices> = OnceCell::uninit();

fn devices<'a>() -> impl Iterator<Item = &'a Arc<Mutex<PciDevice>>> {
    DEVICES
        .get_or_init(|| {
            let devices = unsafe { raw::iterate_all() }
                .map(|v| Arc::new(Mutex::new(v)))
                .collect::<Vec<_>>();
            Devices { devices }
        })
        .iter()
}

pub struct PciDriverDescriptor {
    pub name: &'static str,
    pub probe: fn(&PciDevice) -> bool,
    #[allow(clippy::type_complexity)]
    pub init: fn(Weak<Mutex<PciDevice>>) -> Result<(), Box<dyn Error>>,
}

pub struct Devices {
    devices: Vec<Arc<Mutex<PciDevice>>>,
}

impl Devices {
    pub fn iter(&self) -> impl Iterator<Item = &Arc<Mutex<PciDevice>>> {
        self.devices.iter()
    }
}

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
