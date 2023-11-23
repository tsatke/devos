#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;

use linked_list_allocator::LockedHeap;

use std::arch::syscall::{sys_close, sys_exit, sys_mmap, sys_open, sys_read, sys_write, Errno};

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

fn init(heap_start: *mut u8, heap_size: usize) {
    unsafe {
        ALLOCATOR.lock().init(heap_start, heap_size);
    }
}

#[no_mangle]
pub fn _start() -> isize {
    let start = 0x3333_0000_0000;
    let len = 8 * 1024;

    let eno = sys_mmap(start, len, 0x1 | 0x2, 0x2 | 0x8, 0, 0);
    if eno.as_isize() < 0 {
        sys_exit(-eno.as_isize());
    }

    init(start as *mut u8, len);

    do_something();

    let stdout = must(sys_open("/dev/stdout", 0, 0));
    let greeting = must(sys_open("/var/data/hello.txt", 0, 0));
    let mut data = vec![0_u8; 14];
    let n_read = must(sys_read(greeting, &mut data[0..13]));
    data[13] = b'\n';
    let n_write = must(sys_write(stdout, &data));
    assert_eq!(n_read, n_write - 1);

    sys_close(stdout);
    sys_close(greeting);

    sys_exit(0);
}

fn must(errno: Errno) -> usize {
    if errno.as_isize() < 0 {
        sys_exit(-errno.as_isize());
    }
    errno.as_isize() as usize
}

fn do_something() {
    let mut v = Vec::new();
    for i in 0..10 {
        v.push(i);
    }
}

#[cfg(not(test))]
#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    sys_exit(2)
}
