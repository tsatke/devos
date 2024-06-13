use alloc::ffi::CString;

pub use kernel_api::syscall::Errno;
use kernel_api::syscall::Syscall;

use crate::arch::syscall::{syscall1, syscall3};
use crate::arch::syscall::syscall6;

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

pub fn sys_mmap(
    addr: usize,
    len: usize,
    prot: usize,
    flags: usize,
    fd: usize,
    offset: usize,
) -> Errno {
    unsafe { syscall6(Syscall::Mmap, addr, len, prot, flags, fd, offset) }.into()
}

pub fn sys_exit(status: isize) -> ! {
    unsafe { syscall1(Syscall::Exit, status as usize) };
    unreachable!()
}
