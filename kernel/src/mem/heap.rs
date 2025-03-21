use crate::mem::address_space::{virt_addr_from_page_table_indices, AddressSpace};
use crate::mem::phys::PhysicalMemory;
use crate::U64Ext;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering::Relaxed;
use log::info;
use x86_64::structures::paging::page::PageRangeInclusive;
use x86_64::structures::paging::{Page, PageSize, PageTableFlags, Size4KiB};
use x86_64::VirtAddr;

static HEAP_INITIALIZED: AtomicBool = AtomicBool::new(false);
static HEAP_START: VirtAddr = virt_addr_from_page_table_indices([257, 0, 0, 0], 0);
static INITIAL_HEAP_SIZE: usize = 1024 * 1024; // 1MiB
pub static FINAL_HEAP_SIZE: usize = 32 * 1024 * 1024; // 32MiB

#[global_allocator]
static ALLOCATOR: linked_list_allocator::LockedHeap = linked_list_allocator::LockedHeap::empty();

pub(in crate::mem) fn init(address_space: &AddressSpace) {
    assert!(PhysicalMemory::is_initialized());

    info!("initializing heap at {:p}", HEAP_START);
    let page_range = PageRangeInclusive {
        start: Page::<Size4KiB>::containing_address(HEAP_START),
        end: Page::<Size4KiB>::containing_address(HEAP_START + INITIAL_HEAP_SIZE as u64),
    };

    address_space
        .map_range(
            page_range,
            PhysicalMemory::allocate_frames_non_contiguous(),
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
        )
        .expect("should be able to map heap");

    unsafe {
        ALLOCATOR
            .lock()
            .init(HEAP_START.as_mut_ptr(), INITIAL_HEAP_SIZE);
    }

    HEAP_INITIALIZED.store(true, Relaxed);
}

// In stage2, we already have the physical memory manager that uses the heap, which is much faster
// than the one we use on boot, so we allocate the largest portion of memory for the heap in stage2.
pub(in crate::mem) fn init_stage2() {
    assert!(HEAP_INITIALIZED.load(Relaxed));

    let new_start = HEAP_START + INITIAL_HEAP_SIZE as u64 + Size4KiB::SIZE;

    let page_range = PageRangeInclusive {
        start: Page::<Size4KiB>::containing_address(new_start),
        end: Page::<Size4KiB>::containing_address(
            new_start + (FINAL_HEAP_SIZE - INITIAL_HEAP_SIZE) as u64,
        ),
    };

    let address_space = AddressSpace::kernel();
    address_space
        .map_range(
            page_range,
            PhysicalMemory::allocate_frames(page_range.len().into_usize())
                .expect("should have enough contiguous physical memory for heap"),
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
        )
        .expect("should be able to map more heap");

    unsafe {
        ALLOCATOR.lock().extend(FINAL_HEAP_SIZE - INITIAL_HEAP_SIZE);
    }
}

#[derive(Copy, Clone)]
pub struct Heap;

impl Heap {
    pub fn is_initialized() -> bool {
        HEAP_INITIALIZED.load(Relaxed)
    }

    pub fn free() -> usize {
        ALLOCATOR.lock().free()
    }

    pub fn used() -> usize {
        ALLOCATOR.lock().used()
    }

    pub fn size() -> usize {
        ALLOCATOR.lock().size()
    }

    pub fn bottom() -> VirtAddr {
        VirtAddr::new(ALLOCATOR.lock().bottom() as u64)
    }
}

impl core::fmt::Debug for Heap {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("Heap")
            .field("initialized", &Self::is_initialized())
            .field("free", &Self::free())
            .field("used", &Self::used())
            .field("size", &Self::size())
            .finish()
    }
}
