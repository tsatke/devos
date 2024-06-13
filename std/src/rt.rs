use linked_list_allocator::LockedHeap;

use crate::arch::syscall::{Errno, sys_exit, sys_mmap, sys_open};

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

fn init(heap_start: *mut u8, heap_size: usize) {
    unsafe {
        ALLOCATOR.lock().init(heap_start, heap_size);
    }
}

pub fn start() {
    init_heap();
    init_fds();
}

fn init_heap() {
    let start = 0x3333_0000_0000;
    let len = 8 * 1024;

    let eno = sys_mmap(start, len, 0x1 | 0x2, 0x2 | 0x8, 0, 0);
    if eno.as_isize() < 0 {
        sys_exit(-eno.as_isize());
    }

    init(start as *mut u8, len);
}

fn init_fds() {
    let stdout = must(sys_open("/dev/stdout", 0, 0));
    assert_eq!(stdout, 0); // FIXME: posix mandates that stdout is 1, not 0
}

fn must(errno: Errno) -> usize {
    if errno.as_isize() < 0 {
        sys_exit(-errno.as_isize());
    }
    errno.as_isize() as usize
}