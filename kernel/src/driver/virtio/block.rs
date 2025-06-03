use crate::U64Ext;
use crate::driver::KernelDeviceId;
use crate::driver::block::BlockDevices;
use crate::driver::pci::device::PciDevice;
use crate::driver::pci::{PCI_DRIVERS, PciDriverDescriptor, PciDriverType};
use crate::driver::virtio::hal::{HalImpl, transport};
use alloc::boxed::Box;
use alloc::sync::Arc;
use core::error::Error;
use core::fmt::{Debug, Formatter};
use kernel_device::Device;
use kernel_device::block::{BlockBuf, BlockDevice};
use linkme::distributed_slice;
use spin::Mutex;
use virtio_drivers::device::blk::VirtIOBlk;
use virtio_drivers::transport::pci::PciTransport;
use virtio_drivers::transport::{DeviceType, Transport};

#[distributed_slice(PCI_DRIVERS)]
static VIRTIO_BLK: PciDriverDescriptor = PciDriverDescriptor {
    name: "virtio-blk",
    typ: PciDriverType::Specific,
    probe: virtio_probe,
    init: virtio_init,
};

fn virtio_probe(device: &PciDevice) -> bool {
    device.vendor_id() == 0x1af4
        && (0x1000..=0x103f).contains(&device.device_id())
        && transport(device).device_type() == DeviceType::Block
}

#[allow(clippy::needless_pass_by_value)] // signature is required like this
fn virtio_init(device: PciDevice) -> Result<(), Box<dyn Error>> {
    let transport = transport(&device);

    let blk = VirtIOBlk::<HalImpl, _>::new(transport)?;

    let device = VirtioBlockDevice {
        id: KernelDeviceId::new(),
        inner: Arc::new(Mutex::new(blk)),
    };

    BlockDevices::register_block_device(device)?;
    Ok(())
}

#[derive(Clone)]
pub struct VirtioBlockDevice {
    id: KernelDeviceId,
    inner: Arc<Mutex<VirtIOBlk<HalImpl, PciTransport>>>,
}

impl Debug for VirtioBlockDevice {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("VirtioBlockDevice")
            .field("id", &self.id)
            .finish_non_exhaustive()
    }
}

impl Device<KernelDeviceId> for VirtioBlockDevice {
    fn id(&self) -> KernelDeviceId {
        self.id
    }
}

impl BlockDevice<KernelDeviceId, 512> for VirtioBlockDevice {
    fn block_count(&self) -> usize {
        self.inner.lock().capacity().into_usize()
    }

    fn read_block(
        &mut self,
        block_num: usize,
        buf: &mut BlockBuf<512>,
    ) -> Result<(), Box<dyn Error>> {
        self.inner.lock().read_blocks(block_num, &mut buf[..])?;
        Ok(())
    }

    fn write_block(&mut self, block_num: usize, buf: &BlockBuf<512>) -> Result<(), Box<dyn Error>> {
        self.inner.lock().write_blocks(block_num, &buf[..])?;
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Box<dyn Error>> {
        todo!()
    }
}

impl filesystem::BlockDevice for VirtioBlockDevice {
    type Error = ();

    fn sector_size(&self) -> usize {
        512
    }

    fn sector_count(&self) -> usize {
        self.inner.lock().capacity().into_usize()
    }

    fn read_sector(&self, sector_index: usize, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.inner
            .lock()
            .read_blocks(sector_index, buf)
            .map(|()| buf.len())
            .map_err(|_| ())
    }

    fn write_sector(&mut self, sector_index: usize, buf: &[u8]) -> Result<usize, Self::Error> {
        self.inner
            .lock()
            .write_blocks(sector_index, buf)
            .map(|()| buf.len())
            .map_err(|_| ())
    }
}
