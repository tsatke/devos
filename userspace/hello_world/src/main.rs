#![no_std]
#![no_main]

use std::arch::syscall::sys_exit;

#[no_mangle]
pub fn _start() -> isize {
    sys_exit(1);
}

#[cfg(not(test))]
#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
