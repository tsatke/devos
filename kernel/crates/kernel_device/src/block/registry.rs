use alloc::collections::BTreeMap;
use alloc::sync::Arc;

use spin::RwLock;

use crate::block::BlockDevice;
use crate::{DeviceId, RegisterDeviceError};

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
    pub fn register_device<D>(
        &mut self,
        device: D,
    ) -> Result<Arc<RwLock<D>>, RegisterDeviceError<D>>
    where
        D: BlockDevice<Id, N>,
        D: 'static,
    {
        let id = device.id();
        if self.devices.contains_key(&id) {
            return Err(RegisterDeviceError::AlreadyRegistered(device));
        }

        let res = Arc::new(RwLock::new(device));
        self.devices.insert(id, res.clone());
        Ok(res)
    }

    pub fn all_devices(&self) -> impl Iterator<Item = &Arc<RwLock<dyn BlockDevice<Id, N>>>> {
        self.devices.values()
    }
}
