#![no_std]

extern crate alloc;

use crate::syscall::sys_exit;

pub mod arch;
pub mod rt;
pub mod print;
pub mod syscall;

#[cfg(not(test))]
#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    if let Some(location) = info.location() {
        println!(
            "thread '{}' panicked at {}:{}:{}:\n{}",
            "unknown", // TODO: get the current thread name
            location.file(),
            location.line(),
            location.column(),
            info.message(),
        );
    }
    sys_exit(2)
}