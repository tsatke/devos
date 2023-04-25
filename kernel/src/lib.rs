#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use bootloader_api::BootInfo;
use x86_64::instructions::interrupts;

use crate::arch::{gdt, idt};

pub mod arch;
pub mod mem;
pub mod qemu;

pub fn kernel_init(boot_info: &'static mut BootInfo) {
    gdt::init();
    idt::init();
    mem::init(boot_info);
    interrupts::enable();
}
