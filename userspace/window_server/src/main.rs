#![no_std]
#![no_main]

extern crate alloc;

use std::{println, rt};
use std::arch::syscall::{Errno, sys_exit};

#[no_mangle]
pub fn _start() -> isize {
    rt::start();

    main();

    sys_exit(0);
}

fn must(errno: Errno) -> usize {
    if errno.as_isize() < 0 {
        sys_exit(-errno.as_isize());
    }
    errno.as_isize() as usize
}

fn main() {
    println!("Hello, world!");
}

#[cfg(not(test))]
#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    sys_exit(2)
}
