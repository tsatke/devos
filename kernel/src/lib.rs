#![no_std]
#![no_main]
#![feature(abi_x86_interrupt, naked_functions, negative_impls)]
extern crate alloc;

mod acpi;
mod arch;
pub mod hpet;
pub mod limine;
mod log;
pub mod mem;
mod serial;

pub fn init() {
    log::init();
    arch::init_no_heap();
    mem::init();
    arch::init_with_heap();
    acpi::init();
    hpet::init();
}
