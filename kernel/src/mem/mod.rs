use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::sync::Arc;

use bootloader_api::info::MemoryRegionKind;
use bootloader_api::BootInfo;
use spin::RwLock;
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::{Page, PageSize, PageTableFlags, PhysFrame, Size4KiB};
use x86_64::{PhysAddr, VirtAddr};

pub use address_space::*;
pub use size::*;
use virt::heap::{HEAP_SIZE, HEAP_START};

use crate::mem::virt::{
    heap, Interval, MemoryBackedVmObject, PhysicalAllocationStrategy, PmObject,
};
use crate::process::vmm;
use crate::{process, serial_println, KERNEL_CODE_LEN};
use crate::{Result, KERNEL_CODE_ADDR};

mod address_space;
mod physical;
mod size;
pub mod virt;

pub fn init(boot_info: &'static BootInfo) -> Result<()> {
    physical::init_stage1(boot_info);

    let recursive_index = boot_info.recursive_index.into_option().unwrap();
    let (pt_phys_addr, cr3flags) = Cr3::read();

    let mut address_space = AddressSpace::new(pt_phys_addr, cr3flags, recursive_index);

    // **This is extremely dangerous, only modify if you know what you do!!!**
    // TODO: remove the two-stage approach
    // The address space needs an initialized stage1 memory manager, but if we manage to pass this
    // iterator as the frame allocator, we can remove the two-stage approach. This sounds easier
    // than it is - keep in mind, we don't have heap memory.
    //
    // We can do this because the stage 1 allocator skips the frames required for the heap,
    // so we can do this directly here. This prevents us from having to create the iterator for
    // every frame allocation (which takes multiple seconds per MiB of mapped heap).
    let mut usable_frames = boot_info
        .memory_regions
        .iter()
        // get usable regions from memory map
        .filter(|r| r.kind == MemoryRegionKind::Usable)
        // map each region to its address range
        .map(|r| r.start..r.end)
        // transform to an iterator of frame start addresses
        .flat_map(|r| r.step_by(Page::<Size4KiB>::SIZE as usize))
        // create `PhysFrame` types from the start addresses
        .map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)));

    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
    (HEAP_START..=HEAP_START + HEAP_SIZE.bytes())
        .step_by(Size4KiB::SIZE as usize)
        .map(|v| VirtAddr::new(v as u64))
        .map(Page::<Size4KiB>::containing_address)
        .for_each(|p| unsafe {
            let frame = usable_frames.next().unwrap();
            address_space.map_to(p, frame, flags).unwrap().flush();
        });

    serial_println!(
        "mapped {} kernel heap from {:#p} to {:#p}",
        HEAP_SIZE,
        HEAP_START as *mut (),
        (HEAP_START + HEAP_SIZE.bytes()) as *mut ()
    );

    // after the full heap memory has been mapped, we can init
    unsafe { heap::init(HEAP_START as *mut u8, HEAP_SIZE.bytes()) };

    // after we have heap, we can now switch to the stage 2 physical memory manager
    physical::init_stage2(boot_info);

    serial_println!(
        "{} MiB of initial {} MiB kernel heap available after switching to physical memory management stage 2",
        heap::free() / 1024 / 1024,
        heap::size() / 1024 / 1024,
    );

    // this pm_object shouldn't allocate anything, and it also shouldn't try to free anything on drop
    let zero_pmo = Arc::new(RwLock::new(PmObject::create(
        0,
        PhysicalAllocationStrategy::AllocateOnAccess,
    )?));
    let kheap_start_addr = VirtAddr::new(HEAP_START as u64);
    let kheap_size = HEAP_SIZE.bytes();
    let interval = Interval::new(kheap_start_addr, kheap_size);

    process::init(address_space);

    let vmm = vmm();
    let interval = vmm.mark_as_reserved(interval)?;
    let kheap_vm_object =
        MemoryBackedVmObject::new("kernel_heap".to_string(), zero_pmo.clone(), interval, flags);
    vmm.vm_objects()
        .write()
        .insert(kheap_start_addr, Box::new(kheap_vm_object)); // this needs to happen after we've initialized the heap

    let kernel_code_addr = *KERNEL_CODE_ADDR
        .get()
        .expect("kernel code address not initialized");
    let kernel_code_len = *KERNEL_CODE_LEN
        .get()
        .expect("kernel code length not initialized");
    let interval = Interval::new(kernel_code_addr, kernel_code_len);
    let interval = vmm.mark_as_reserved(interval)?;
    let kcode_vm_object =
        MemoryBackedVmObject::new("kernel_code".to_string(), zero_pmo.clone(), interval, flags);
    vmm.vm_objects()
        .write()
        .insert(kernel_code_addr, Box::new(kcode_vm_object));

    Ok(())
}

/// Map a physical frame to a page in the current address space.
#[macro_export]
macro_rules! map_page {
    ($page:expr, $size:ident, $flags:expr) => {{
        let frame = $crate::mem::PhysicalMemoryManager::lock()
            .allocate_frame()
            .unwrap();
        map_page!($page, frame, $size, $flags)
    }};
    ($page:expr, $frame:expr, $size:ident, $flags:expr) => {{
        let page: Page<$size> = $page;
        let process = $crate::process::current();
        let mut address_space = process.address_space().write();
        unsafe { address_space.map_to(page, $frame, $flags).unwrap().flush() }
    }};
}

#[macro_export]
macro_rules! unmap_page {
    ($page:expr, $size:ident) => {{
        let page: Page<$size> = $page;
        let process = $crate::process::current();
        let mut address_space = process.address_space().write();
        address_space.unmap(page).unwrap().1.flush()
    }};
}
