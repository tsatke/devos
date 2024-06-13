#![no_std]
#![no_main]

extern crate alloc;

use kernel_api::syscall::{SocketDomain, SocketType};
use std::{println, rt};
use std::syscall::{sys_exit, sys_socket};

#[no_mangle]
pub fn _start() -> isize {
    rt::start();

    main();

    sys_exit(0);
}

fn main() {
    sys_socket(SocketDomain::Unix, SocketType::Stream, 0);
    println!("Hello, world!");
}
