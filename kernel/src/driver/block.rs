use alloc::format;
use alloc::sync::Arc;

use kernel_devfs::BlockDeviceFile;
use kernel_device::block::registry::BlockDeviceRegistry;
use kernel_device::block::BlockDevice;
use kernel_device::RegisterDeviceError;
use kernel_vfs::path::AbsoluteOwnedPath;
use spin::RwLock;

use crate::driver::KernelDeviceId;
use crate::file::devfs::devfs;

static BLOCK_DEVICES: RwLock<BlockDeviceRegistry<KernelDeviceId, 512>> =
    RwLock::new(BlockDeviceRegistry::new());

pub struct BlockDevices;

impl BlockDevices {
    // TODO: add documentation
    #[allow(clippy::missing_errors_doc)]
    #[allow(clippy::missing_panics_doc)]
    pub fn register_block_device<D>(device: Arc<RwLock<D>>) -> Result<(), RegisterDeviceError>
    where
        D: BlockDevice<KernelDeviceId, 512> + Send + Sync + 'static,
    {
        BLOCK_DEVICES.write().register_device(device.clone())?;

        let id = device.read().id();
        let path = AbsoluteOwnedPath::try_from(format!("/blk{id}").as_ref()).unwrap();
        devfs()
            .write()
            .register_file(path.as_ref(), {
                move || Ok(BlockDeviceFile::new(device.clone()))
            })
            .unwrap();

        Ok(())
    }
}
