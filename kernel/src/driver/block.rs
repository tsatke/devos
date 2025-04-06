use crate::driver::KernelDeviceId;
use device::block::registry::BlockDeviceRegistry;
use device::block::BlockDevice;
use device::RegisterDeviceError;
use spin::RwLock;

static BLOCK_DEVICES: RwLock<BlockDeviceRegistry<KernelDeviceId, 512>> =
    RwLock::new(BlockDeviceRegistry::new());

pub struct BlockDevices;

impl BlockDevices {
    #[allow(clippy::missing_errors_doc)] // TODO: add documentation
    pub fn register_block_device<D>(device: D) -> Result<(), RegisterDeviceError<D>>
    where
        D: BlockDevice<KernelDeviceId, 512> + 'static,
    {
        BLOCK_DEVICES.write().register_device(device)?;
        // TODO: create a vfs entry for this device
        Ok(())
    }
}
