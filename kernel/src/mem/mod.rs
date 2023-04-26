use bootloader_api::BootInfo;
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::{Page, PageSize, PageTableFlags, Size4KiB};
use x86_64::VirtAddr;

pub use address_space::*;
pub use heap::*;
pub use manager::*;
pub use size::*;

use crate::serial_println;

mod address_space;
mod heap;
mod manager;
pub mod physical;
mod size;

const HEAP_START: usize = 0x4444_4444_0000;
const HEAP_SIZE: Size = Size::MiB(2);

pub fn init(boot_info: &'static mut BootInfo) {
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

    let _new_address_space = AddressSpace::allocate_new(&mut address_space);
}
