use core::sync::atomic::AtomicU64;
use core::sync::atomic::Ordering::Relaxed;

use kernel_device::DeviceId;

pub mod block;
pub mod pci;
pub mod raw;
pub mod vga;
pub mod virtio;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct KernelDeviceId(u64);

impl Default for KernelDeviceId {
    fn default() -> Self {
        Self::new()
    }
}

impl KernelDeviceId {
    pub fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        Self(COUNTER.fetch_add(1, Relaxed))
    }
}

impl DeviceId for KernelDeviceId {}
