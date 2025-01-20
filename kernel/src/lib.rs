#![no_std]
#![no_main]
#![feature(abi_x86_interrupt, naked_functions)]
extern crate alloc;

mod arch;
mod log;
mod mem;
mod serial;

pub fn init() {
    log::init();
    arch::init_no_heap();
    mem::init();
    arch::init_with_heap();
}
