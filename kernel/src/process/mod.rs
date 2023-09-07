mod process;
mod scheduler;
mod task;
mod tree;

pub use process::*;
pub use scheduler::*;
pub use tree::*;

pub fn current() -> Process {
    unsafe { scheduler() }.current_process().clone()
}
