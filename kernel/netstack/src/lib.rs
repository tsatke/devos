#![no_std]
extern crate alloc;

use crate::executor::Executor;

pub mod executor;
pub mod net;

pub struct NetStack {
    executor: Executor,
}
