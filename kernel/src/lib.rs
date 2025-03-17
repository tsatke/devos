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
pub mod smp;

pub fn init() {
    log::init();
    mem::init();
    acpi::init();
    hpet::init();
}
