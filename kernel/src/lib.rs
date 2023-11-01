#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(allocator_api)]
#![feature(array_chunks)]
#![feature(assert_matches)]
#![feature(const_mut_refs)]
#![feature(exclusive_range_pattern)]
#![feature(iter_array_chunks)]
#![feature(naked_functions)]
#![feature(negative_impls)]
#![feature(never_type)]

extern crate alloc;

use bootloader_api::config::Mapping;
use bootloader_api::{BootInfo, BootloaderConfig};
use x86_64::instructions::interrupts;

use crate::arch::{gdt, idt};
use crate::io::vfs;
use crate::mem::Size;

pub mod acpi;
pub mod apic;
pub mod arch;
pub mod io;
pub mod mem;
pub mod process;
pub mod qemu;
pub mod screen;
pub mod syscall;
pub mod timer;

const KERNEL_STACK_SIZE: Size = Size::KiB(128);

pub const fn bootloader_config() -> BootloaderConfig {
    let mut config = BootloaderConfig::new_default();
    config.mappings.page_table_recursive = Some(Mapping::Dynamic);
    config.mappings.framebuffer = Mapping::Dynamic;
    config.kernel_stack_size = KERNEL_STACK_SIZE.bytes() as u64;
    config
}

#[allow(clippy::needless_pass_by_ref_mut)]
pub fn kernel_init(boot_info: &'static mut BootInfo) {
    gdt::init();
    idt::init();
    mem::init(boot_info);
    acpi::init(boot_info);
    apic::init();
    vfs::init();

    let fb = boot_info.framebuffer.as_ref().unwrap();
    let fb_info = fb.info();
    screen::init(fb.buffer().as_ptr(), fb_info);

    interrupts::enable();
}

#[cfg(feature = "kernel_test")]
mod tests {
    use kernel_test_framework::kernel_test;

    #[kernel_test]
    fn test_it_works() {
        assert_eq!(2 + 2, 4);
    }
}
