use crate::UsizeExt;
use crate::driver::KernelDeviceId;
use crate::driver::pci::device::PciDevice;
use crate::driver::pci::{PCI_DRIVERS, PciDriverDescriptor, PciDriverType};
use crate::driver::raw::RawDevices;
use crate::driver::virtio::hal::{HalImpl, transport};
use crate::mem::address_space::AddressSpace;
use alloc::boxed::Box;
use alloc::sync::Arc;
use core::error::Error;
use core::fmt::{Debug, Formatter};
use kernel_device::Device;
use kernel_device::raw::RawDevice;
use linkme::distributed_slice;
use spin::Mutex;
use virtio_drivers::device::gpu::VirtIOGpu;
use virtio_drivers::transport::pci::PciTransport;
use virtio_drivers::transport::{DeviceType, Transport};
use x86_64::VirtAddr;
use x86_64::structures::paging::frame::PhysFrameRangeInclusive;
use x86_64::structures::paging::{PhysFrame, Size4KiB};

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

    let fb = gpu.setup_framebuffer()?;
    let buffer_virtual_addr = VirtAddr::from_ptr(fb);
    let buffer_len = fb.len();
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

    let phys_addr = AddressSpace::kernel()
        .translate(buffer_virtual_addr)
        .expect("address should be mapped into kernel space");
    let start = PhysFrame::<Size4KiB>::containing_address(phys_addr);
    let end = PhysFrame::<Size4KiB>::containing_address(phys_addr + buffer_len.into_u64() - 1);
    let physical_memory = PhysFrameRangeInclusive { start, end };

    let device = VirtioRawDevice {
        id: KernelDeviceId::new(),
        _inner: Arc::new(Mutex::new(gpu)),
        physical_memory,
    };

    RawDevices::register_raw_device(device)?;

    Ok(())
}

#[derive(Clone)]
pub struct VirtioRawDevice {
    id: KernelDeviceId,
    _inner: Arc<Mutex<VirtIOGpu<HalImpl, PciTransport>>>,
    physical_memory: PhysFrameRangeInclusive,
}

impl Debug for VirtioRawDevice {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("VirtioRawDevice")
            .field("id", &self.id)
            .field("physical_memory", &self.physical_memory)
            .finish_non_exhaustive()
    }
}

impl Device<KernelDeviceId> for VirtioRawDevice {
    fn id(&self) -> KernelDeviceId {
        self.id
    }
}

impl RawDevice<KernelDeviceId> for VirtioRawDevice {
    fn physical_memory(&self) -> PhysFrameRangeInclusive {
        self.physical_memory
    }
}
