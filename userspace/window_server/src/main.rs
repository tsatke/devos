#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::ToString;
use core::slice::from_raw_parts_mut;

use kernel_api::syscall::{FfiSockAddr, SocketDomain, SocketType, Stat};
use std::syscall::{sys_bind, sys_close, sys_exit, sys_mmap, sys_open, sys_socket, sys_stat, Errno};
use std::{println, rt};

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

    let mut stat = Stat::default();
    let res = sys_stat("/dev/fb0", &mut stat);
    if res == Errno::ENOENT {
        println!("No framebuffer found");
        return;
    }
    res.unwrap();

    println!("framebuffer stat: {:?}", stat);

    let fd = sys_open("/dev/fb0", 0, 0).unwrap();
    let addr = sys_mmap(0, stat.size as usize, 3, 2, fd, 0).unwrap();
    sys_close(fd).unwrap();
    let fb = unsafe { from_raw_parts_mut(addr as *mut u32, stat.size as usize / 4) };
    fb.fill(0x0000_FF00);

    const WIDTH: usize = 1280;
    #[allow(dead_code)]
    const HEIGHT: usize = 800;

    for v in (0x00..0xFF).chain((0x00..0xFF).rev()).cycle() {
        for _ in 0..5 {
            fb
                .chunks_exact_mut(WIDTH)
                .skip(200)
                .take(80)
                .flat_map(|row| row.iter_mut().skip(400).take(80))
                .for_each(|pixel| *pixel = 0xFF - (v / 2) << 8 | v);
        }
    }
}