#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;

use std::{println, rt};
use std::syscall::{Errno, sys_close, sys_exit, sys_open, sys_read};

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
    let mut v = Vec::new();
    for i in 0..10 {
        v.push(i);
    }

    let greeting = must(sys_open("/var/data/hello.txt", 0, 0));
    let mut data = vec![0_u8; 13];
    let n_read = must(sys_read(greeting, &mut data[0..13]));
    assert_eq!(n_read, 13);
    println!("hello.txt contained: '{}'", core::str::from_utf8(&data).unwrap());

    sys_close(greeting);
}
