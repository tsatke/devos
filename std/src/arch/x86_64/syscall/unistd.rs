use crate::arch::syscall::syscall1;
use kernel_api::syscall::Syscall;

pub fn sys_exit(status: isize) -> ! {
    unsafe { syscall1(Syscall::Exit, status as usize) };
    unreachable!()
}
