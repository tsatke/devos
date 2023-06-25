#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(assert_matches)]
#![feature(const_mut_refs)]
#![feature(naked_functions)]
#![feature(negative_impls)]

use bootloader_api::config::Mapping;
use bootloader_api::{BootInfo, BootloaderConfig};
use x86_64::instructions::interrupts;

use crate::arch::{gdt, idt};
use crate::mem::Size;

extern crate alloc;

pub mod acpi;
pub mod apic;
pub mod arch;
pub mod mem;
pub mod process;
pub mod qemu;
pub mod screen;
pub mod timer;

#[cfg(not(any(feature = "bios", feature = "uefi")))]
compile_error!("You must enable either the bios or uefi feature");
#[cfg(all(feature = "bios", feature = "uefi"))]
compile_error!("You must enable either the bios or uefi feature, not both");

const KERNEL_STACK_SIZE: Size = Size::KiB(32);

pub const fn bootloader_config() -> BootloaderConfig {
    let mut config = BootloaderConfig::new_default();
    config.mappings.page_table_recursive = Some(Mapping::Dynamic);
    config.mappings.framebuffer = Mapping::FixedAddress(0xa0000);
    config.kernel_stack_size = KERNEL_STACK_SIZE.bytes() as u64;
    config
}

pub fn kernel_init(boot_info: &'static mut BootInfo) {
    gdt::init();
    idt::init();
    mem::init(boot_info);
    acpi::init(boot_info);
    apic::init();

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
        fn compute() -> usize {
            2 + 2
        }
        assert_eq!(4, compute());
    }
}
