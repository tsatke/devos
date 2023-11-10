#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec::Vec;

use linked_list_allocator::LockedHeap;

use std::arch::syscall::{sys_exit, sys_mmap};

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

fn init(heap_start: *mut u8, heap_size: usize) {
    unsafe {
        ALLOCATOR.lock().init(heap_start, heap_size);
    }
}

#[no_mangle]
pub fn _start() -> isize {
    let start = 0x1111_1111_0000;
    let len = 8 * 1024;

    let eno = sys_mmap(start, len, 0x1 | 0x2, 0x2 | 0x8, 0, 0);
    if eno.as_isize() < 0 {
        sys_exit(-eno.as_isize());
    }

    init(start as *mut u8, len);

    do_something();

    sys_exit(0);
}

fn do_something() {
    let mut v = Vec::new();
    for i in 0..(4 * 1024) {
        v.push(i);
    }
}

#[cfg(not(test))]
#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    sys_exit(2)
}
