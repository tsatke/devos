#![no_std]
#![no_main]
#![feature(abi_x86_interrupt, negative_impls)]
extern crate alloc;

use crate::driver::pci;
use crate::limine::BOOT_TIME;
use conquer_once::spin::OnceCell;
use ::log::info;

mod acpi;
mod apic;
mod arch;
pub mod backtrace;
pub mod driver;
pub mod hpet;
pub mod limine;
mod log;
pub mod mcore;
pub mod mem;
mod serial;
pub mod syscall;
pub mod time;
pub mod vfs;

static BOOT_TIME_SECONDS: OnceCell<u64> = OnceCell::uninit();

/// # Panics
/// Panics if there was no boot time provided by limine.
fn init_boot_time() {
    BOOT_TIME_SECONDS.init_once(|| BOOT_TIME.get_response().unwrap().timestamp().as_secs());
}

pub fn init() {
    init_boot_time();

    log::init();
    mem::init();
    acpi::init();
    apic::init();
    hpet::init();
    backtrace::init();
    mcore::init();
    pci::init();

    info!("kernel initialized");
}

#[cfg(target_pointer_width = "64")]
pub trait U64Ext {
    fn into_usize(self) -> usize;
}

#[cfg(target_pointer_width = "64")]
impl U64Ext for u64 {
    #[allow(clippy::cast_possible_truncation)]
    fn into_usize(self) -> usize {
        // Safety: we know that we are on 64-bit, so this is correct
        unsafe { usize::try_from(self).unwrap_unchecked() }
    }
}

#[cfg(target_pointer_width = "64")]
pub trait UsizeExt {
    fn into_u64(self) -> u64;
}

#[cfg(target_pointer_width = "64")]
impl UsizeExt for usize {
    fn into_u64(self) -> u64 {
        self as u64
    }
}
