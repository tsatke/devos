#![no_std]
#![no_main]

extern crate alloc;

use std::rt;
use std::syscall::sys_exit;

#[no_mangle]
pub fn _start() -> isize {
    rt::start();

    main();

    sys_exit(0);
}

fn main() {
    panic!("Hello, world!");
}
