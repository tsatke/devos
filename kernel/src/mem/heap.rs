use crate::mem::address_space::AddressSpace;
use crate::mem::phys::PhysicalMemory;
use crate::mem::virt_addr_from_page_table_indices;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering::Relaxed;
use log::info;
use x86_64::structures::paging::{Page, PageSize, PageTableFlags, Size4KiB};
use x86_64::VirtAddr;

static HEAP_INITIALIZED: AtomicBool = AtomicBool::new(false);
static HEAP_START: VirtAddr = virt_addr_from_page_table_indices(&[257, 0, 0, 0], 0);
static INITIAL_HEAP_SIZE: usize = 5 * 1024 * 1024; // 1 MiB

#[global_allocator]
static ALLOCATOR: linked_list_allocator::LockedHeap = linked_list_allocator::LockedHeap::empty();

pub(in crate::mem) fn init(address_space: AddressSpace) {
    assert!(PhysicalMemory::is_initialized());
    let mut address_space = address_space;

    info!("initializing heap at {:p}", HEAP_START);
    for vaddr in
        (HEAP_START..HEAP_START + INITIAL_HEAP_SIZE as u64).step_by(Size4KiB::SIZE as usize)
    {
        let page = Page::<Size4KiB>::containing_address(vaddr);
        let frame =
            PhysicalMemory::allocate_frame().expect("should have enough memory to allocate heap");
        address_space
            .map(
                page,
                frame,
                PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
            )
            .expect("should be able to map heap");
    }

    unsafe {
        ALLOCATOR
            .lock()
            .init(HEAP_START.as_mut_ptr(), INITIAL_HEAP_SIZE);
    }
    HEAP_INITIALIZED.store(true, Relaxed);
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
