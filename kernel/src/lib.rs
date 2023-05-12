#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(assert_matches)]
#![feature(const_mut_refs)]
#![feature(naked_functions)]
#![feature(negative_impls)]

use bootloader_api::BootInfo;
use x86_64::instructions::interrupts;

use crate::arch::{gdt, idt};

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

pub fn kernel_init(boot_info: &'static mut BootInfo) {
    gdt::init();
    idt::init();
    mem::init(boot_info);
    acpi::init(boot_info);
    apic::init();

    let fb = boot_info.framebuffer.as_ref().unwrap();
    let fb_info = fb.info();
    screen::init(fb.buffer().as_ptr() as *const u8, fb_info);

    interrupts::enable();
}
