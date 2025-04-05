use crate::driver::block::{register_block_device, BlockDevice};
use crate::driver::pci::device::PciDevice;
use crate::driver::pci::{PciDriverDescriptor, PciDriverType, PCI_DRIVERS};
use crate::driver::virtio::hal::{transport, HalImpl};
use alloc::boxed::Box;
use core::error::Error;
use linkme::distributed_slice;
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

fn virtio_init(device: PciDevice) -> Result<(), Box<dyn Error>> {
    let transport = transport(&device);

    let blk = VirtIOBlk::<HalImpl, _>::new(transport).unwrap();
    let device = VirtioBlockDevice { inner: blk };
    register_block_device(device);

    Ok(())
}

pub struct VirtioBlockDevice {
    inner: VirtIOBlk<HalImpl, PciTransport>,
}

impl BlockDevice for VirtioBlockDevice {
    fn read(&mut self, block_id: usize, buf: &mut [u8]) {
        self.inner.read_blocks(block_id, buf).unwrap();
    }
}
