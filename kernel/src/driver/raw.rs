use crate::driver::KernelDeviceId;
use kernel_device::RegisterDeviceError;
use kernel_device::raw::{RawDevice, RawDeviceRegistry};
use spin::RwLock;

static RAW_DEVICES: RwLock<RawDeviceRegistry<KernelDeviceId>> =
    RwLock::new(RawDeviceRegistry::new());

pub struct RawDevices;

impl RawDevices {
    #[allow(clippy::missing_errors_doc)] // TODO: add documentation
    pub fn register_raw_device<D>(device: D) -> Result<(), RegisterDeviceError<D>>
    where
        D: RawDevice<KernelDeviceId> + 'static,
    {
        RAW_DEVICES.write().register_device(device)?;
        // TODO: create a vfs entry for this device
        Ok(())
    }
}
