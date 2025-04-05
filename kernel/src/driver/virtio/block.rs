use crate::driver::pci::device::PciDevice;
use crate::driver::pci::{PciDriverDescriptor, PciDriverType, PCI_DRIVERS};
use crate::driver::virtio::hal::{transport, HalImpl};
use crate::vfs::ext2::VirtualExt2Fs;
use crate::vfs::vfs;
use crate::U64Ext;
use alloc::boxed::Box;
use alloc::sync::Arc;
use core::error::Error;
use core::str::from_utf8;
use ext2::Ext2Fs;
use linkme::distributed_slice;
use log::info;
use spin::Mutex;
use vfs::path::ROOT;
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
    device.vendor_id() == 0x1AF4
        && (0x1000..=0x103f).contains(&device.device_id())
        && transport(device).device_type() == DeviceType::Block
}

#[allow(clippy::needless_pass_by_value)] // signature is required like this
fn virtio_init(device: PciDevice) -> Result<(), Box<dyn Error>> {
    let transport = transport(&device);

    let mut blk = VirtIOBlk::<HalImpl, _>::new(transport)?;
    let mut device_id_buf = [0_u8; 20];
    let read = blk.device_id(&mut device_id_buf)?;
    let device_id = from_utf8(&device_id_buf[..read])?;

    let device = VirtioBlockDevice {
        inner: Arc::new(Mutex::new(blk)),
    };

    // TODO: one block device can have multiple partitions, support that

    let ext2fs = Ext2Fs::try_new(device)?;
    let wrapped = VirtualExt2Fs::from(ext2fs);
    info!("mounting device with id '{device_id}' and type 'ext2' at '{ROOT}'");
    vfs().write().mount(ROOT, wrapped)?;

    Ok(())
}

#[derive(Clone)]
pub struct VirtioBlockDevice {
    inner: Arc<Mutex<VirtIOBlk<HalImpl, PciTransport>>>,
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
