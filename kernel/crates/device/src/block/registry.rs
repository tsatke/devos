use crate::block::BlockDevice;
use crate::DeviceId;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use spin::RwLock;
use thiserror::Error;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Error)]
pub enum RegisterDeviceError<D> {
    #[error("device id is already registered")]
    AlreadyRegistered(D),
}

pub struct BlockDeviceRegistry<Id, const N: usize>
where
    Id: DeviceId + Ord + 'static,
{
    devices: BTreeMap<Id, Arc<RwLock<dyn BlockDevice<Id, N>>>>,
}

unsafe impl<Id, const N: usize> Send for BlockDeviceRegistry<Id, N> where
    Id: DeviceId + Ord + 'static
{
}

unsafe impl<Id, const N: usize> Sync for BlockDeviceRegistry<Id, N> where
    Id: DeviceId + Ord + 'static
{
}

impl<Id, const N: usize> Default for BlockDeviceRegistry<Id, N>
where
    Id: DeviceId + Ord + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<Id, const N: usize> BlockDeviceRegistry<Id, N>
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

impl<Id, const N: usize> BlockDeviceRegistry<Id, N>
where
    Id: DeviceId + Ord + 'static,
{
    /// # Errors
    /// Returns an error if the device is already registered, returning the
    /// device that could not be registered.
    pub fn register_device<D>(&mut self, device: D) -> Result<(), RegisterDeviceError<D>>
    where
        D: BlockDevice<Id, N>,
        D: 'static,
    {
        if self.devices.contains_key(&device.id()) {
            return Err(RegisterDeviceError::AlreadyRegistered(device));
        }

        self.devices
            .insert(device.id(), Arc::new(RwLock::new(device)));
        Ok(())
    }

    pub fn all_devices(&self) -> impl Iterator<Item = &Arc<RwLock<dyn BlockDevice<Id, N>>>> {
        self.devices.values()
    }
}
