#![no_std]
#![no_main]

use core::arch::asm;
use core::ffi::{c_char, c_int};

unsafe extern "C" {
    fn exit(code: c_int) -> !;

    fn open(file: *const c_char, oflag: c_int, ...) -> c_int;
    fn read(fd: c_int, buf: *mut u8, count: usize) -> isize;
    fn write(fd: c_int, buf: *const u8, count: usize) -> isize;

    fn errno_location() -> *mut c_int;
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() {
    const STDOUT: c_int = 1;

    // FIXME: find out why this is not aligned in the first place
    // align stack to 16 bytes
    unsafe {
        asm! {
            "push rbp",
            "mov rbp, rsp",
            "and rsp, -16",
        };
    }

    unsafe {
        let hello_txt_path = b"/var/hello.txt\0";
        let hello_txt = open(hello_txt_path.as_ptr().cast(), 0, 0);
        if hello_txt < 0 {
            exit(errno_location().read());
        }
        let mut buf = [0u8; 128];
        let n = read(hello_txt, buf.as_mut_ptr(), buf.len());
        if n < 0 {
            exit(errno_location().read());
        }
        let data = &buf[..n as usize];

        let v = write(STDOUT, data.as_ptr(), data.len());
        if v < 0 {
            exit(errno_location().read());
        }

        exit(errno_location().read());
    }
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &::core::panic::PanicInfo) -> ! {
    unsafe { exit(1) };
}
