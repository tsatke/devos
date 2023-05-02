use bootloader_api::BootInfo;
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::{Page, PageSize, PageTableFlags, Size4KiB};
use x86_64::VirtAddr;

pub use address_space::*;
pub use heap::*;
pub use manager::*;
pub use size::*;

use crate::process::task::Task;
use crate::{process, serial_println};

mod address_space;
mod heap;
mod manager;
pub mod physical;
mod size;

const HEAP_START: usize = 0x4444_4444_0000;
const HEAP_SIZE: Size = Size::MiB(2);

pub fn init(boot_info: &'static BootInfo) {
    manager::init(boot_info);

    let recursive_index = boot_info.recursive_index.into_option().unwrap();
    let (pt_phys_addr, cr3flags) = Cr3::read();

    let mut address_space = AddressSpace::new(pt_phys_addr, cr3flags, recursive_index);

    (HEAP_START..=HEAP_START + HEAP_SIZE.bytes())
        .step_by(Size4KiB::SIZE as usize)
        .map(|v| VirtAddr::new(v as u64))
        .map(Page::<Size4KiB>::containing_address)
        .for_each(|p| unsafe {
            let frame = MemoryManager::lock().allocate_frame().unwrap();
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
