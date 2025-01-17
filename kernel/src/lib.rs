#![no_std]
#![no_main]
#![feature(abi_x86_interrupt, naked_functions)]
extern crate alloc;

use crate::arch::gdt;

mod arch;
mod log;
mod mem;
mod serial;

pub fn init() {
    log::init();
    gdt::init();
    mem::init();
}
