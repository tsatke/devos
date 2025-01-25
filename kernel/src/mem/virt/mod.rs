use crate::limine::HHDM_REQUEST;
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

    {
        // limine maps 4GiB at HHDM start + 0x00

        // the kernel is part of those 4GiB, so we don't need to mark it as reserved (as long as it's smaller than 4GiB)

        let hhdm = HHDM_REQUEST
            .get_response()
            .expect("should have a HHDM response");
        let hhdm_start = VirtAddr::new(hhdm.offset());
        let hhdm_segment = Segment::new(hhdm_start, 4 * 1024 * 1024 * 1024);
        VirtualMemory::mark_as_reserved(hhdm_segment).expect("HHDM should not be reserved yet");
    }

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
