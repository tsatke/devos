use alloc::collections::BTreeMap;
use alloc::sync::Arc;

use spin::RwLock;
use x86_64::structures::paging::frame::PhysFrameRangeInclusive;

use crate::{Device, DeviceId, RegisterDeviceError};

pub trait RawDevice<Id: DeviceId>: Device<Id> {
    fn physical_memory(&self) -> PhysFrameRangeInclusive;
}

pub struct RawDeviceRegistry<Id>
where
    Id: DeviceId + Ord + 'static,
{
    devices: BTreeMap<Id, Arc<RwLock<dyn RawDevice<Id>>>>,
}

unsafe impl<Id> Send for RawDeviceRegistry<Id> where Id: DeviceId + Ord + 'static {}

unsafe impl<Id> Sync for RawDeviceRegistry<Id> where Id: DeviceId + Ord + 'static {}

impl<Id> Default for RawDeviceRegistry<Id>
where
    Id: DeviceId + Ord + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<Id> RawDeviceRegistry<Id>
where
    Id: DeviceId + Ord + 'static,
{
    #[must_use]
    pub const fn new() -> Self {
        Self {
            devices: BTreeMap::new(),
        }
    }
}

impl<Id> RawDeviceRegistry<Id>
where
    Id: DeviceId + Ord + 'static,
{
    /// # Errors
    /// Returns an error if the device is already registered, returning the
    /// device that could not be registered.
    pub fn register_device<D>(&mut self, device: D) -> Result<(), RegisterDeviceError<D>>
    where
        D: RawDevice<Id>,
        D: 'static,
    {
        if self.devices.contains_key(&device.id()) {
            return Err(RegisterDeviceError::AlreadyRegistered(device));
        }

        self.devices
            .insert(device.id(), Arc::new(RwLock::new(device)));
        Ok(())
    }

    pub fn all_devices(&self) -> impl Iterator<Item = &Arc<RwLock<dyn RawDevice<Id>>>> {
        self.devices.values()
    }
}
