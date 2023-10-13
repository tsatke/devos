#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(array_chunks)]
#![feature(assert_matches)]
#![feature(const_mut_refs)]
#![feature(exclusive_range_pattern)]
#![feature(iter_array_chunks)]
#![feature(naked_functions)]
#![feature(negative_impls)]

use bootloader_api::BootInfo;
use x86_64::instructions::interrupts;

use crate::arch::{gdt, idt};
use crate::io::vfs;

extern crate alloc;

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

#[cfg(not(any(feature = "bios", feature = "uefi")))]
compile_error!("You must enable either the bios or uefi feature");
#[cfg(all(feature = "bios", feature = "uefi"))]
compile_error!("You must enable either the bios or uefi feature, not both");

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
