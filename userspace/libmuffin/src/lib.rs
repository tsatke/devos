//! `libmuffin` is the implementation of the [`POSIX.1-2024`] standard that is used by the
//! Muffin operating system.
//!
//! It provides the basic system calls and libraries that are expected to be available in a
//! POSIX-compliant environment, such as file control operations, process management, signal handling,
//! and standard input/output operations.
//!
//! [`POSIX.1-2024`]: https://pubs.opengroup.org/onlinepubs/9799919799

#![no_std]
#![allow(unused_variables)] // TODO: remove
#![feature(c_variadic, linkage, thread_local)]

extern crate compiler_builtins;
extern crate unwinding;

pub mod errno;
pub mod fcntl;
pub mod poll;
pub mod pthread;
pub mod signal;
pub mod stdlib;
pub mod string;
pub mod sys;
pub mod syscall;
pub mod unistd;

use kernel_abi::SYS_EXIT;
use libc::c_int;

fn unimplemented_function(n: usize) -> ! {
    syscall::syscall1(n, 0);
    unreachable!();
}

#[linkage = "weak"]
#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    exit(-1);
}

#[unsafe(no_mangle)]
pub extern "C" fn exit(result: c_int) -> ! {
    unimplemented_function(SYS_EXIT);
}
