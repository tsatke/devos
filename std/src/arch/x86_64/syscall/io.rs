use alloc::ffi::CString;

use kernel_api::syscall::{Errno, Syscall};

use crate::arch::syscall::{syscall1, syscall3};

pub fn sys_read(fd: usize, buf: &mut [u8]) -> Errno {
    unsafe { syscall3(Syscall::Read, fd, buf.as_mut_ptr() as usize, buf.len()) }.into()
}

pub fn sys_write(fd: usize, buf: &[u8]) -> Errno {
    unsafe { syscall3(Syscall::Write, fd, buf.as_ptr() as usize, buf.len()) }.into()
}

pub fn sys_open(path: &str, flags: usize, mode: usize) -> Errno {
    let cstring = CString::new(path).unwrap();
    unsafe { syscall3(Syscall::Open, cstring.as_ptr() as usize, flags, mode) }.into()
}

pub fn sys_close(fd: usize) -> Errno {
    unsafe { syscall1(Syscall::Close, fd) }.into()
}
