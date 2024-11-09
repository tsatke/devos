use alloc::ffi::CString;
use core::ptr::addr_of;

pub use kernel_api::syscall::Errno;
use kernel_api::syscall::{FfiSockAddr, SocketDomain, SocketType, Stat, Syscall};

use crate::arch::syscall::syscall6;
use crate::arch::syscall::{syscall1, syscall2, syscall3};

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

pub fn sys_socket(domain: SocketDomain, ty: SocketType, protocol: usize) -> Errno {
    unsafe { syscall3(Syscall::Socket, domain as usize, ty as usize, protocol) }.into()
}

pub fn sys_bind(socket: usize, address: FfiSockAddr, address_len: usize) -> Errno {
    unsafe {
        syscall3(
            Syscall::Bind,
            socket,
            addr_of!(address) as usize,
            address_len,
        )
    }
    .into()
}

pub fn sys_stat(path: &str, stat: &mut Stat) -> Errno {
    let cstring = CString::new(path).unwrap();
    unsafe {
        syscall2(
            Syscall::Stat,
            cstring.as_ptr() as usize,
            stat as *mut Stat as usize,
        )
    }
    .into()
}
