use crate::driver::pci::device::PciDevice;
use crate::driver::pci::raw::iterate_all;
use alloc::boxed::Box;
use alloc::string::ToString;
use core::error::Error;
use linkme::distributed_slice;
use log::{debug, error, log_enabled, trace, warn, Level};

pub use raw::PortCam;

pub mod device;
mod raw;
pub mod register;

#[distributed_slice]
pub static PCI_DRIVERS: [PciDriverDescriptor] = [..];

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PciDriverType {
    Generic,
    Specific,
}

pub struct PciDriverDescriptor {
    pub name: &'static str,
    pub typ: PciDriverType,
    pub probe: fn(&PciDevice) -> bool,
    pub init: fn(PciDevice) -> Result<(), Box<dyn Error>>,
}

/// # Panics
///
/// Panics if there are multiple specific or multiple generic drivers that would match
/// the same device.
pub fn init() {
    if log_enabled!(Level::Trace) {
        PCI_DRIVERS
            .iter()
            .for_each(|driver| trace!("have pci driver: {}", driver.name));
    }

    unsafe { iterate_all() }.for_each(|device| {
        let driver = PCI_DRIVERS
            .iter()
            .fold(None, |res: Option<&PciDriverDescriptor>, driver| {
                if !(driver.probe)(&device) {
                    return res;
                }

                if let Some(other_driver) = res {
                    if other_driver.typ == PciDriverType::Generic
                        && driver.typ == PciDriverType::Specific
                    {
                        return Some(driver);
                    } else if other_driver.typ == PciDriverType::Specific
                        && driver.typ == PciDriverType::Generic
                    {
                        return Some(other_driver);
                    }

                    panic!(
                        "found two drivers for the same device: {} and {}",
                        other_driver.name, driver.name
                    );
                } else {
                    Some(driver)
                }
            });
        if let Some(driver) = driver {
            debug!("found driver {} for device {}", driver.name, device);
            let device_string = device.to_string();
            if let Err(e) = (driver.init)(device) {
                error!(
                    "failed to init driver {} for device {}: {}",
                    driver.name, device_string, e
                );
            }
        } else {
            warn!("no driver found for device {device}");
        }
    });
}
