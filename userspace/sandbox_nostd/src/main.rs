#![no_std]
#![no_main]

use core::ffi::{c_char, c_int};

unsafe extern "C" {
    fn exit(code: c_int) -> !;

    fn open(file: *const c_char, oflag: c_int, ...) -> c_int;
    fn read(fd: c_int, buf: *mut u8, count: usize) -> isize;
    fn write(fd: c_int, buf: *const u8, count: usize) -> isize;
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() {
    const STDOUT: c_int = 1;

    unsafe {
        let hello_txt_path = b"/var/hello.txt\0";
        let hello_txt = open(hello_txt_path.as_ptr().cast(), 0, 0);
        let mut buf = [0u8; 128];
        let n = read(hello_txt, buf.as_mut_ptr(), buf.len()) as usize;
        let data = &buf[..n];

        write(STDOUT, data.as_ptr(), data.len());

        exit(0);
    }
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &::core::panic::PanicInfo) -> ! {
    unsafe { exit(1) };
}
