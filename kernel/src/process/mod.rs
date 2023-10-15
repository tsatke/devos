use crate::mem::AddressSpace;
use crate::process::task::{Ready, Running, Task};
use alloc::string::String;
use alloc::sync::Arc;
use core::sync::atomic::AtomicU64;
use core::sync::atomic::Ordering::Relaxed;
use derive_more::Display;
use spin::{RwLock, RwLockReadGuard, RwLockWriteGuard};

pub mod elf;
mod scheduler;
mod task;
mod tree;

pub use scheduler::*;
pub use tree::*;

pub fn create(parent: Process, name: impl Into<String>) -> Process {
    let address_space = AddressSpace::allocate_new();
    let process = Process::new(name, address_space);

    let process_tree = unsafe { process_tree_mut() };
    process_tree.insert_process(parent, process.clone());

    process
}

pub fn dump_tree() {
    // TODO: remove
    let process_tree = unsafe { process_tree_mut() };
    process_tree.dump();
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
    unsafe { spawn(task) }
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
    name: String,
    inner: Arc<RwLock<ProcessData>>,
}

impl Process {
    pub fn new(name: impl Into<String>, address_space: AddressSpace) -> Self {
        let data = ProcessData::new(address_space);
        Self {
            id: ProcessId::new(),
            name: name.into(),
            inner: Arc::new(RwLock::new(data)),
        }
    }

    pub fn process_id(&self) -> &ProcessId {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn read(&self) -> RwLockReadGuard<ProcessData> {
        self.inner.read()
    }

    pub fn write(&self) -> RwLockWriteGuard<ProcessData> {
        self.inner.write()
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct ProcessData {
    address_space: AddressSpace,
}

impl ProcessData {
    fn new(address_space: AddressSpace) -> Self {
        Self { address_space }
    }

    pub fn address_space(&self) -> &AddressSpace {
        &self.address_space
    }

    pub fn address_space_mut(&mut self) -> &mut AddressSpace {
        &mut self.address_space
    }
}
