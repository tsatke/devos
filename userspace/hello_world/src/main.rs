#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;

use std::arch::syscall::{Errno, sys_close, sys_exit, sys_open, sys_read, sys_write};
use std::rt;

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

    // write something to stdout

    let stdout = must(sys_open("/dev/stdout", 0, 0));
    let greeting = must(sys_open("/var/data/hello.txt", 0, 0));
    let mut data = vec![0_u8; 14];
    let n_read = must(sys_read(greeting, &mut data[0..13]));
    data[13] = b'\n';
    let n_write = must(sys_write(stdout, &data));
    assert_eq!(n_read, n_write - 1);

    sys_close(stdout);
    sys_close(greeting);
}

#[cfg(not(test))]
#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    sys_exit(2)
}
