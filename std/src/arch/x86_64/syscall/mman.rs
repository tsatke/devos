use kernel_api::syscall::{Errno, Syscall};

use crate::arch::syscall::syscall6;

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
