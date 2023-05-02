#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(const_mut_refs)]
#![feature(naked_functions)]

use bootloader_api::BootInfo;
use x86_64::instructions::interrupts;

use crate::arch::{gdt, idt};

extern crate alloc;

pub mod arch;
pub mod mem;
pub mod process;
pub mod qemu;

#[cfg(not(any(feature = "bios", feature = "uefi")))]
compile_error!("You must enable either the bios or uefi feature");
#[cfg(all(feature = "bios", feature = "uefi"))]
compile_error!("You must enable either the bios or uefi feature, not both");

pub fn kernel_init(boot_info: &'static mut BootInfo) {
    gdt::init();
    idt::init();
    mem::init(boot_info);
    interrupts::enable();
}
