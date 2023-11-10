use core::arch::asm;

pub use io::*;
pub use kernel_api::syscall::Errno;
use kernel_api::syscall::Syscall;
pub use mman::*;
pub use unistd::*;

mod io;
mod mman;
mod unistd;

/// # Safety
/// Depending on the syscall, the caller must ensure that all arguments are valid.
pub unsafe fn syscall1(syscall: Syscall, arg1: usize) -> isize {
    let res: isize;
    asm! {
        "int 0x80",
        in("rax") syscall as usize,
        in("rdi") arg1,
        lateout("rax") res,
    }
    res
}

/// # Safety
/// Depending on the syscall, the caller must ensure that all arguments are valid.
pub unsafe fn syscall2(syscall: Syscall, arg1: usize, arg2: usize) -> isize {
    let res: isize;
    asm! {
        "int 0x80",
        in("rax") syscall as usize,
        in("rdi") arg1,
        in("rsi") arg2,
        lateout("rax") res,
    }
    res
}

/// # Safety
/// Depending on the syscall, the caller must ensure that all arguments are valid.
pub unsafe fn syscall3(syscall: Syscall, arg1: usize, arg2: usize, arg3: usize) -> isize {
    let res: isize;
    asm! {
        "int 0x80",
        in("rax") syscall as usize,
        in("rdi") arg1,
        in("rsi") arg2,
        in("rdx") arg3,
        lateout("rax") res,
    }
    res
}

/// # Safety
/// Depending on the syscall, the caller must ensure that all arguments are valid.
pub unsafe fn syscall4(
    syscall: Syscall,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
) -> isize {
    let res: isize;
    asm! {
        "int 0x80",
        in("rax") syscall as usize,
        in("rdi") arg1,
        in("rsi") arg2,
        in("rdx") arg3,
        in("rcx") arg4,
        lateout("rax") res,
    }
    res
}

/// # Safety
/// Depending on the syscall, the caller must ensure that all arguments are valid.
pub unsafe fn syscall5(
    syscall: Syscall,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
    arg5: usize,
) -> isize {
    let res: isize;
    asm! {
        "int 0x80",
        in("rax") syscall as usize,
        in("rdi") arg1,
        in("rsi") arg2,
        in("rdx") arg3,
        in("rcx") arg4,
        in("r8") arg5,
        lateout("rax") res,
    }
    res
}

/// # Safety
/// Depending on the syscall, the caller must ensure that all arguments are valid.
pub unsafe fn syscall6(
    syscall: Syscall,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
    arg5: usize,
    arg6: usize,
) -> isize {
    let res: isize;
    asm! {
        "int 0x80",
        in("rax") syscall as usize,
        in("rdi") arg1,
        in("rsi") arg2,
        in("rdx") arg3,
        in("rcx") arg4,
        in("r8") arg5,
        in("r9") arg6,
        lateout("rax") res,
    }
    res
}
