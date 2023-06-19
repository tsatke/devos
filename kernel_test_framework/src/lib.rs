#![no_std]
#![feature(error_in_core)]
#![deny(unsafe_op_in_unsafe_fn)]

pub use derive::kernel_test;

use linkme::distributed_slice;

/// Function signature used for kernel test functions
pub type KernelTestFn = fn();

/// The source code location of a test function
#[derive(Debug, Clone)]
pub struct SourceLocation {
    /// module of the source location
    pub module: &'static str,
    /// file of the source location
    pub file: &'static str,
    /// line of the source location
    pub line: u32,
    /// column of the source location
    pub column: u32,
}

impl core::fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{}:{}:{}", self.file, self.line, self.column))
    }
}

/// Description of a single kernel test
#[derive(Debug, Clone)]
pub struct KernelTestDescription {
    pub name: &'static str,
    pub fn_name: &'static str,
    pub test_fn: KernelTestFn,
    pub test_location: SourceLocation,
}

#[distributed_slice]
pub static KERNEL_TESTS: [KernelTestDescription] = [..];
