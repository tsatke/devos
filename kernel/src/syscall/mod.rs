use access::KernelAccess;
use core::ops::Neg;
use kernel_abi::syscall_name;
use kernel_abi::Errno;
use kernel_syscall::fcntl::sys_open;
use kernel_syscall::unistd::sys_getcwd;
use kernel_syscall::{UserspaceMutPtr, UserspacePtr};
use log::{error, trace};
use x86_64::instructions::hlt;

mod access;

#[must_use]
pub fn dispatch_syscall(
    n: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
    arg5: usize,
    arg6: usize,
) -> isize {
    trace!("syscall: {n} {arg1} {arg2} {arg3} {arg4} {arg5} {arg6}");

    let result = match n {
        kernel_abi::SYS_OPEN => dispatch_sys_open(arg1, arg2, arg3),
        kernel_abi::SYS_GETCWD => dispatch_sys_getcwd(arg1, arg2),
        _ => {
            error!("unimplemented syscall: {} ({n})", syscall_name(n));
            loop {
                hlt();
            }
        }
    };

    match result {
        Ok(ret) => {
            trace!("syscall {n} returned {ret}");
            ret as isize
        }
        Err(e) => {
            error!("syscall {n} failed with error: {e:?}");
            Into::<isize>::into(e).neg()
        }
    }
}

fn dispatch_sys_getcwd(path: usize, size: usize) -> Result<usize, Errno> {
    let cx = KernelAccess::new();

    let path = unsafe { UserspaceMutPtr::try_from_usize(path)? };
    sys_getcwd(&cx, path, size)
}

fn dispatch_sys_open(path: usize, oflag: usize, mode: usize) -> Result<usize, Errno> {
    let cx = KernelAccess::new();

    let path = unsafe { UserspacePtr::try_from_usize(path)? };
    sys_open(&cx, path, oflag as i32, mode as i32)
}
