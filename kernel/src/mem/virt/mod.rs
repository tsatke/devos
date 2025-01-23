use crate::limine::KERNEL_FILE_REQUEST;
use crate::mem::heap::Heap;
use conquer_once::spin::OnceCell;
use core::mem::ManuallyDrop;
use spin::RwLock;
use virtual_memory_manager::{AlreadyReserved, Segment, VirtualMemoryManager};
use x86_64::VirtAddr;

static VMM: OnceCell<RwLock<VirtualMemoryManager>> = OnceCell::uninit();

fn vmm() -> &'static RwLock<VirtualMemoryManager> {
    VMM.get().expect("virtual memory should be initialized")
}

pub fn init() {
    VMM.init_once(|| {
        RwLock::new(VirtualMemoryManager::new(
            VirtAddr::new(0xFFFF_8000_0000_0000),
            0x0000_7FFF_FFFF_FFFF,
        ))
    });

    let response = KERNEL_FILE_REQUEST
        .get_response()
        .expect("should have a kernel file response");
    let kernel_mapping = Segment::new(
        VirtAddr::from_ptr(response.file().addr()),
        response.file().size(),
    );
    VirtualMemory::mark_as_reserved(kernel_mapping)
        .expect("kernel file should not be reserved yet");
    VirtualMemory::mark_as_reserved(Segment::new(Heap::bottom(), Heap::size() as u64))
        .expect("heap should not be reserved yet");
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct OwnedSegment {
    inner: Segment,
}

impl OwnedSegment {
    #[allow(dead_code)]
    pub fn leak(self) -> Segment {
        ManuallyDrop::new(self).inner
    }
}

impl Drop for OwnedSegment {
    fn drop(&mut self) {
        VirtualMemory::release(self);
    }
}

pub struct VirtualMemory;

impl VirtualMemory {
    #[allow(dead_code)]
    pub fn reserve(n: usize) -> Option<OwnedSegment> {
        vmm()
            .write()
            .reserve(n)
            .map(|segment| OwnedSegment { inner: segment })
    }

    pub fn mark_as_reserved(segment: Segment) -> Result<(), AlreadyReserved> {
        vmm().write().mark_as_reserved(segment)
    }

    fn release(segment: &mut OwnedSegment) -> bool {
        vmm().write().release(segment.inner)
    }
}
