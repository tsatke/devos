#![no_std]
#![no_main]

extern crate alloc;

use std::arch::syscall::sys_exit;
use std::rt;

#[no_mangle]
pub fn _start() -> isize {
    rt::start();

    main();

    sys_exit(0);
}

fn main() {
    panic!("Hello, world!");
}
