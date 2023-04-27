#![no_std]
#![feature(start)]

use core::panic::PanicInfo;
use std::syscall::sys_write;

#[start]
fn start(_argc: isize, _argv: *const *const u8) -> isize {
    main();
    0
}

#[panic_handler]
fn panic_handler(_info: &PanicInfo) -> ! {
    todo!()
}

fn main() {
    let _ = sys_write(0, &[1, 2, 3]);
}
