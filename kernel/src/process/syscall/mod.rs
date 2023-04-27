use kernel_api::syscall::Syscall;

use crate::process::syscall::io::*;
use crate::serial_println;

pub mod io;

pub fn dispatch_syscall(
    syscall: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
) -> isize {
    let syscall = TryInto::<Syscall>::try_into(syscall);
    if syscall.is_err() {
        return -1;
    }
    let syscall = syscall.unwrap();

    match syscall {
        Syscall::Read => sys_read(arg1, arg2 as *mut u8, arg3),
        Syscall::Write => sys_write(arg1, arg2 as *mut u8, arg3),
        Syscall::Open => sys_open(arg1 as *const u8, arg2, arg3),
    }
}
