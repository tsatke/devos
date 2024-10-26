#![no_std]
#![feature(int_roundings)]
#![feature(allocator_api)]
extern crate alloc;

pub mod syscall;

pub const PATH_MAX: usize = 4096;
