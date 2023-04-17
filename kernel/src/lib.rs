#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

pub mod arch;
pub mod qemu;

use crate::arch::{gdt, idt};
use bootloader_api::BootInfo;
use x86_64::instructions::interrupts;

pub fn kernel_init(_boot_info: &'static mut BootInfo) {
    gdt::init();
    idt::init();
    interrupts::enable();
}
