#![allow(dead_code)] // TODO: remove

use core::arch::asm;

/// Perform a system call with a single argument.
///
/// This function is intended to be used for making a system call
/// to the kernel, and is not POSIX compliant, meaning that it
/// does not modify `errno` on failure.
///
/// If you use this, you must
/// handle the return value and any errors yourself. This includes
/// emulating behavior that POSIX specifies.
pub(crate) fn syscall1(number: usize, arg1: usize) -> isize {
    let result: isize;
    unsafe {
        asm!(
        "int 0x80",
        in("rax") number,
        in("rdi") arg1,
        lateout("rax") result,
        );
    }
    result
}

/// Perform a system call with two arguments.
///
/// This function is intended to be used for making a system call
/// to the kernel, and is not POSIX compliant, meaning that it
/// does not modify `errno` on failure.
///
/// If you use this, you must
/// handle the return value and any errors yourself. This includes
/// emulating behavior that POSIX specifies.
pub(crate) fn syscall2(number: usize, arg1: usize, arg2: usize) -> isize {
    let result: isize;
    unsafe {
        asm!(
        "int 0x80",
        in("rax") number,
        in("rdi") arg1,
        in("rsi") arg2,
        lateout("rax") result,
        );
    }
    result
}

/// Perform a system call with three arguments.
///
/// This function is intended to be used for making a system call
/// to the kernel, and is not POSIX compliant, meaning that it
/// does not modify `errno` on failure.
///
/// If you use this, you must
/// handle the return value and any errors yourself. This includes
/// emulating behavior that POSIX specifies.
pub(crate) fn syscall3(number: usize, arg1: usize, arg2: usize, arg3: usize) -> isize {
    let result: isize;
    unsafe {
        asm!(
        "int 0x80",
        in("rax") number,
        in("rdi") arg1,
        in("rsi") arg2,
        in("rdx") arg3,
        lateout("rax") result,
        );
    }
    result
}

/// Perform a system call with four arguments.
///
/// This function is intended to be used for making a system call
/// to the kernel, and is not POSIX compliant, meaning that it
/// does not modify `errno` on failure.
///
/// If you use this, you must
/// handle the return value and any errors yourself. This includes
/// emulating behavior that POSIX specifies.
pub(crate) fn syscall4(number: usize, arg1: usize, arg2: usize, arg3: usize, arg4: usize) -> isize {
    let result: isize;
    unsafe {
        asm!(
        "int 0x80",
        in("rax") number,
        in("rdi") arg1,
        in("rsi") arg2,
        in("rdx") arg3,
        in("rcx") arg4,
        lateout("rax") result,
        );
    }
    result
}

/// Perform a system call with five arguments.
///
/// This function is intended to be used for making a system call
/// to the kernel, and is not POSIX compliant, meaning that it
/// does not modify `errno` on failure.
///
/// If you use this, you must
/// handle the return value and any errors yourself. This includes
/// emulating behavior that POSIX specifies.
pub(crate) fn syscall5(
    number: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
    arg5: usize,
) -> isize {
    let result: isize;
    unsafe {
        asm!(
        "int 0x80",
        in("rax") number,
        in("rdi") arg1,
        in("rsi") arg2,
        in("rdx") arg3,
        in("rcx") arg4,
        in("r8") arg5,
        lateout("rax") result,
        );
    }
    result
}

/// Perform a system call with six arguments.
///
/// This function is intended to be used for making a system call
/// to the kernel, and is not POSIX compliant, meaning that it
/// does not modify `errno` on failure.
///
/// If you use this, you must
/// handle the return value and any errors yourself. This includes
/// emulating behavior that POSIX specifies.
pub(crate) fn syscall6(
    number: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
    arg5: usize,
    arg6: usize,
) -> isize {
    let result: isize;
    unsafe {
        asm!(
        "int 0x80",
        in("rax") number,
        in("rdi") arg1,
        in("rsi") arg2,
        in("rdx") arg3,
        in("rcx") arg4,
        in("r8") arg5,
        in("r9") arg6,
        lateout("rax") result,
        );
    }
    result
}
