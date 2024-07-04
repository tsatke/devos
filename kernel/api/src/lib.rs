#![no_std]
#![feature(const_option)]
#![feature(int_roundings)]
extern crate alloc;

pub mod syscall;

pub const PATH_MAX: usize = 4096;
