use core::ops::Neg;
use kernel_abi::syscall_name;
use kernel_abi::{Errno, SYS_OPEN};
use log::{error, trace};
use x86_64::instructions::hlt;

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
        SYS_OPEN => dispatch_sys_open(arg1, arg2, arg3),
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

fn dispatch_sys_open(_path: usize, _oflag: usize, _mode: usize) -> Result<usize, Errno> {
    // let path = unsafe { UserspacePtr::try_from_usize(path)? };
    // sys_open(todo!(), path, oflag as i32, mode as i32)
    todo!()
}
