use crate::syscall::{syscall1, syscall3};
use kernel_api::syscall::Syscall;

pub fn sys_read(fd: usize, buf: &mut [u8]) -> isize {
    unsafe { syscall3(Syscall::Read, fd, buf.as_mut_ptr() as usize, buf.len()) }
}

pub fn sys_write(fd: usize, buf: &[u8]) -> isize {
    unsafe { syscall3(Syscall::Write, fd, buf.as_ptr() as usize, buf.len()) }
}

pub fn sys_open(path: &str, flags: usize, mode: usize) -> isize {
    unsafe { syscall3(Syscall::Open, (&path as *const &str) as usize, flags, mode) }
}

pub fn sys_close(fd: usize) -> isize {
    unsafe { syscall1(Syscall::Close, fd) }
}
