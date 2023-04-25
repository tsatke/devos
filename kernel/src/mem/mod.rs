use bootloader_api::BootInfo;
use core::ptr;
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::{Page, PageTableFlags};
use x86_64::VirtAddr;

use crate::serial_println;
pub use address_space::*;
pub use manager::*;
pub use size::*;

mod address_space;
mod manager;
pub mod physical;
mod size;

pub fn init(boot_info: &'static mut BootInfo) {
    manager::init(boot_info);

    let recursive_index = boot_info.recursive_index.into_option().unwrap();
    let (pt_phys_addr, cr3flags) = Cr3::read();

    let mut address_space = AddressSpace::new(pt_phys_addr, cr3flags, recursive_index);

    let new_frame = MemoryManager::lock().allocate_frame().unwrap();
    let new_addr = VirtAddr::new(0x5555_5555_0000);
    unsafe {
        address_space
            .map_to(
                Page::containing_address(new_addr),
                new_frame,
                PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
            )
            .unwrap()
            .flush();
    }

    unsafe {
        serial_println!("old: {}", ptr::read_volatile::<u64>(new_addr.as_ptr()));
        ptr::write_volatile::<u64>(new_addr.as_mut_ptr(), 1234567890);
        serial_println!("new: {}", ptr::read_volatile::<u64>(new_addr.as_ptr()));
    }

    let _new_address_space = AddressSpace::allocate_new(&mut address_space);
}
