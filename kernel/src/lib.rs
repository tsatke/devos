#![no_std]
#![no_main]
#![feature(abi_x86_interrupt, naked_functions, negative_impls)]
extern crate alloc;

use crate::hpet::hpet;
use crate::limine::BOOT_TIME;
use conquer_once::spin::OnceCell;
use jiff::Timestamp;

mod acpi;
mod apic;
mod arch;
pub mod hpet;
pub mod limine;
mod log;
pub mod mcore;
pub mod mem;
mod serial;

static BOOT_TIME_SECONDS: OnceCell<u64> = OnceCell::uninit();

pub fn init() {
    BOOT_TIME_SECONDS.init_once(|| BOOT_TIME.get_response().unwrap().timestamp().as_secs());

    log::init();
    mem::init();
    acpi::init();
    apic::init();
    hpet::init();
}

pub fn now() -> Timestamp {
    let counter = hpet().read().main_counter_value();
    let secs = BOOT_TIME_SECONDS.get().unwrap();
    let secs = secs + (counter / 1_000_000_000);
    let ts = Timestamp::new(
        i64::try_from(secs).expect("shouldn't have more seconds than i64::MAX"),
        (counter % 1_000_000_000) as i32,
    )
    .unwrap();
    ts
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
