use crate::driver::KernelDeviceId;
use crate::vfs::ext2::VirtualExt2Fs;
use crate::vfs::vfs;
use alloc::boxed::Box;
use alloc::sync::Arc;
use core::error::Error;
use ext2::Ext2Fs;
use kernel_device::RegisterDeviceError;
use kernel_device::block::registry::BlockDeviceRegistry;
use kernel_device::block::{BlockBuf, BlockDevice};
use kernel_vfs::path::ROOT;
use spin::RwLock;

static BLOCK_DEVICES: RwLock<BlockDeviceRegistry<KernelDeviceId, 512>> =
    RwLock::new(BlockDeviceRegistry::new());

pub struct BlockDevices;

impl BlockDevices {
    // TODO: add documentation
    #[allow(clippy::missing_errors_doc)]
    #[allow(clippy::missing_panics_doc)]
    pub fn register_block_device<D>(device: D) -> Result<(), RegisterDeviceError<D>>
    where
        D: BlockDevice<KernelDeviceId, 512> + Send + Sync + 'static,
    {
        let device = BLOCK_DEVICES.write().register_device(device)?;
        let wrapper = BlockDeviceWrapper { inner: device };
        let fs = Ext2Fs::try_new(wrapper).unwrap();
        let vext2 = VirtualExt2Fs::from(fs);
        vfs().write().mount(ROOT, vext2).unwrap();

        Ok(())
    }
}

struct BlockDeviceWrapper<T, const N: usize>
where
    T: BlockDevice<KernelDeviceId, N>,
{
    inner: Arc<RwLock<T>>,
}

impl<T, const N: usize> filesystem::BlockDevice for BlockDeviceWrapper<T, N>
where
    T: BlockDevice<KernelDeviceId, N>,
{
    type Error = Box<dyn Error>;

    fn sector_size(&self) -> usize {
        N
    }

    fn sector_count(&self) -> usize {
        self.inner.read().block_count()
    }

    fn read_sector(&self, sector_index: usize, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let mut block_buf = BlockBuf::new();
        self.inner
            .write()
            .read_block(sector_index, &mut block_buf)?;
        buf.copy_from_slice(&*block_buf);
        Ok(block_buf.len())
    }

    fn write_sector(&mut self, sector_index: usize, buf: &[u8]) -> Result<usize, Self::Error> {
        let mut block_buf = BlockBuf::new();
        block_buf.copy_from_slice(buf);
        self.inner.write().write_block(sector_index, &block_buf)?;
        Ok(block_buf.len())
    }
}
