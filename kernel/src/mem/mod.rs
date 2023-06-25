use bootloader_api::info::MemoryRegionKind;
use bootloader_api::BootInfo;
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::{Page, PageSize, PageTableFlags, PhysFrame, Size4KiB};
use x86_64::{PhysAddr, VirtAddr};

pub use address_space::*;
pub use heap::*;
pub use manager::*;
pub use size::*;

use crate::process::task::Task;
use crate::{process, serial_println};

mod address_space;
mod heap;
mod manager;
mod physical_stage1;
mod physical_stage2;
mod size;

const HEAP_START: usize = 0x4444_4444_0000;
const HEAP_SIZE: Size = Size::MiB(2);

pub fn init(boot_info: &'static BootInfo) {
    init_stage1(boot_info);

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

    (HEAP_START..=HEAP_START + HEAP_SIZE.bytes())
        .step_by(Size4KiB::SIZE as usize)
        .map(|v| VirtAddr::new(v as u64))
        .map(Page::<Size4KiB>::containing_address)
        .for_each(|p| unsafe {
            let frame = usable_frames.next().unwrap();
            address_space
                .map_to(p, frame, PageTableFlags::PRESENT | PageTableFlags::WRITABLE)
                .unwrap()
                .flush();
        });

    serial_println!(
        "mapped {} kernel heap from {:#p} to {:#p}",
        HEAP_SIZE,
        HEAP_START as *mut (),
        (HEAP_START + HEAP_SIZE.bytes()) as *mut ()
    );

    // after the full heap memory has been mapped, we can init
    heap::init(HEAP_START as *mut u8, HEAP_SIZE.bytes());

    // after we have heap, we can now switch to the stage 2 physical memory manager
    init_stage2(boot_info);

    serial_println!(
        "{} MiB of initial {} MiB kernel heap available after switching to physical memory management stage 2",
        heap::free() / 1024 / 1024,
        heap::size() / 1024 / 1024,
    );

    let task = Task::new(address_space);
    let _ = process::current_task_mut().insert(task);

    // let _new_address_space = AddressSpace::allocate_new(&mut address_space);
}

/// Map a physical frame to a page in the current address space.
#[macro_export]
macro_rules! map_page {
    ($page:expr, $size:ident, $flags:expr) => {{
        let frame = $crate::mem::MemoryManager::lock().allocate_frame().unwrap();
        map_page!($page, frame, $size, $flags)
    }};
    ($page:expr, $frame:expr, $size:ident, $flags:expr) => {{
        let page: Page<$size> = $page;
        let mut task = $crate::process::current_task_mut();
        let address_space = task.as_mut().unwrap().address_space_mut();
        unsafe { address_space.map_to(page, $frame, $flags).unwrap().flush() }
    }};
}

#[cfg(feature = "kernel_test")]
mod tests {
    use crate::mem::HEAP_SIZE;
    use alloc::boxed::Box;
    use core::ptr::write_volatile;
    use kernel_test_framework::kernel_test;

    #[kernel_test]
    fn test_heap_allocation_deallocation() {
        // just making sure that we allocate more that we have available in total, to
        // make sure that the memory is dropped again
        for _ in 0..HEAP_SIZE.bytes() / 512 {
            let mut data = Box::new([0xAB_u8; 1024]);
            data.iter().for_each(|&element| assert_eq!(0xAB, element));
            data.iter_mut()
                .for_each(|element| unsafe { write_volatile(element as *mut u8, 17) });
            data.iter().for_each(|&element| assert_eq!(0x11, element));
        }
    }
}
