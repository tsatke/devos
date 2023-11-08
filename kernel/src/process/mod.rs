use alloc::boxed::Box;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::sync::atomic::AtomicU64;
use core::sync::atomic::Ordering::Relaxed;

use derive_more::Display;
use spin::RwLock;

pub use scheduler::*;
pub use tree::*;

use crate::mem::virt::VmObject;
use crate::mem::AddressSpace;
use crate::process::task::{Ready, Running, Task};

pub mod elf;
mod scheduler;
mod task;
mod tree;

pub fn init(root_process: Process) {
    let current_task = unsafe { Task::kernel_task(root_process.clone()) };
    let mut pt_guard = process_tree().write();
    pt_guard.set_root(root_process.clone());
    pt_guard.add_task(root_process.process_id(), current_task.task_id());

    scheduler::init(current_task);
}

pub fn create(parent: Process, name: impl Into<String>) -> Process {
    let address_space = AddressSpace::allocate_new();
    let process = Process::new(name, address_space);

    process_tree()
        .write()
        .insert_process(parent, process.clone());

    process
}

pub fn current() -> Process {
    current_task().process().clone()
}

pub fn current_task() -> &'static Task<Running> {
    unsafe { scheduler() }.current_task()
}

pub fn spawn_task_in_current_process(name: impl Into<String>, func: extern "C" fn()) {
    spawn_task(name, current(), func)
}

pub fn spawn_task(name: impl Into<String>, process: Process, func: extern "C" fn()) {
    let task = Task::<Ready>::new(process, name, func);
    spawn(task)
}

pub fn exit() -> ! {
    unsafe { exit_current_task() }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Display)]
pub struct ProcessId(u64);

impl !Default for ProcessId {}

impl ProcessId {
    pub fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        ProcessId(COUNTER.fetch_add(1, Relaxed))
    }
}

/// A process is a ref counted rwlock over [`ProcessData`], which is the actual data
/// of the process.
#[derive(Clone, Debug)]
pub struct Process {
    id: ProcessId,
    cr3_value: usize, // TODO: remove this, read it from the address space (maybe use an atomic to circumvent the locking?)
    name: String,
    address_space: Arc<RwLock<AddressSpace>>,
    vm_objects: Arc<RwLock<Vec<Box<dyn VmObject>>>>,
}

impl Process {
    #[allow(clippy::arc_with_non_send_sync)] // FIXME: I don't currently see a way around this
    pub fn new(name: impl Into<String>, address_space: AddressSpace) -> Self {
        Self {
            id: ProcessId::new(),
            cr3_value: address_space.cr3_value(),
            name: name.into(),
            address_space: Arc::new(RwLock::new(address_space)),
            vm_objects: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn process_id(&self) -> &ProcessId {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn address_space(&self) -> &RwLock<AddressSpace> {
        &self.address_space
    }

    pub(in crate::process) fn cr3_value(&self) -> usize {
        self.cr3_value
    }

    pub fn vm_objects(&self) -> &RwLock<Vec<Box<dyn VmObject>>> {
        &self.vm_objects
    }
}
