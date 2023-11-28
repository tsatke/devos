use alloc::string::String;

pub use process::*;
pub use scheduler::*;
pub use tree::*;

use crate::mem::virt::VirtualMemoryManager;
use crate::process::task::{Ready, Running, Task};

pub mod attributes;
pub mod elf;
pub mod fd;
mod process;
mod scheduler;
mod task;
mod tree;

pub fn init(root_process: Process) {
    let current_task = unsafe { Task::kernel_task(root_process.clone()) };
    let mut pt_guard = process_tree().write();
    pt_guard.set_root(root_process.clone());
    pt_guard.add_task(root_process.pid(), current_task.task_id());

    scheduler::init(current_task);
}

// pub fn create(parent: Process, name: impl Into<String>, attributes: Attributes) -> Process {
//     let address_space = AddressSpace::allocate_new();
//     let process = Process::new(name, address_space, attributes);
//
//     process_tree()
//         .write()
//         .insert_process(parent, process.clone());
//
//     process
// }

pub fn current() -> &'static Process {
    current_task().process()
}

pub fn vmm() -> &'static VirtualMemoryManager {
    current().vmm()
}

pub fn current_task() -> &'static Task<Running> {
    unsafe { scheduler() }.current_task()
}

pub fn spawn_task_in_current_process(name: impl Into<String>, func: extern "C" fn()) {
    spawn_task(name, current(), func)
}

pub fn spawn_task(name: impl Into<String>, process: &Process, func: extern "C" fn()) {
    let task = Task::<Ready>::new(process, name, func);
    spawn(task)
}

pub fn exit() -> ! {
    unsafe { exit_current_task() }
}
