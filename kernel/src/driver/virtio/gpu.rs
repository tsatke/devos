use crate::driver::pci::device::PciDevice;
use crate::driver::pci::{PciDriverDescriptor, PciDriverType, PCI_DRIVERS};
use crate::driver::virtio::hal::{transport, HalImpl};
use alloc::boxed::Box;
use core::error::Error;
use linkme::distributed_slice;
use log::info;
use virtio_drivers::device::gpu::VirtIOGpu;
use virtio_drivers::transport::{DeviceType, Transport};

#[distributed_slice(PCI_DRIVERS)]
static VIRTIO_GPU: PciDriverDescriptor = PciDriverDescriptor {
    name: "virtio-gpu",
    typ: PciDriverType::Specific,
    probe: virtio_probe,
    init: virtio_init,
};

fn virtio_probe(device: &PciDevice) -> bool {
    device.vendor_id() == 0x1af4 && transport(device).device_type() == DeviceType::GPU
}

#[allow(clippy::needless_pass_by_value)] // signature is required like this
fn virtio_init(device: PciDevice) -> Result<(), Box<dyn Error>> {
    let transport = transport(&device);

    let mut gpu = VirtIOGpu::<HalImpl, _>::new(transport)?;
    let (width, height) = gpu.resolution()?;
    let width = width as usize;
    let height = height as usize;
    info!("GPU resolution is {width}x{height}");
    let fb = gpu.setup_framebuffer()?;
    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) * 4;

            // truncation intended if happens (which it shouldn't unless we're dealing with really large resolutions
            #[allow(clippy::cast_possible_truncation)]
            {
                fb[idx] = x as u8;
                fb[idx + 1] = y as u8;
                fb[idx + 2] = (x + y) as u8;
            }
        }
    }
    gpu.flush()?;
    Box::leak(Box::new(gpu));

    Ok(())
}
