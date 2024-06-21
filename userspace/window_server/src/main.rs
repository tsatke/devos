#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::ToString;

use kernel_api::syscall::{FfiSockAddr, SocketDomain, SocketType};
use std::{println, rt};
use std::syscall::{sys_bind, sys_exit, sys_socket};

#[no_mangle]
pub fn _start() -> isize {
    rt::start();

    main();

    sys_exit(0);
}

fn main() {
    let socket = sys_socket(SocketDomain::Unix, SocketType::Stream, 0).unwrap();

    // Unimportant detail: If we don't use .to_string() here, the address will be in kernel space
    // because the string is a constant and comes from the binary, not the heap.
    // I guess there is no solid guarantee that to_string will allocate on the heap, but since we
    // don't rely on it and I wanted to see a heap address in the log, I left this here.
    // If you see this, and it bothers you, feel free to change it (as long as you don't break
    // anything).
    let path = "/socket".to_string();

    let address = FfiSockAddr {
        domain: SocketDomain::Unix,
        data: path.as_ptr(),
    };
    let address_len = path.len();
    sys_bind(socket, address, address_len).unwrap();

    println!("Hello, world!");
}