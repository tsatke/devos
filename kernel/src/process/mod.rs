use crate::mem::AddressSpace;
use alloc::sync::Arc;
use core::sync::atomic::AtomicU64;
use core::sync::atomic::Ordering::Relaxed;
use derive_more::Display;
use spin::{RwLock, RwLockReadGuard, RwLockWriteGuard};

mod scheduler;
mod task;
mod tree;

pub use scheduler::*;
pub use tree::*;

pub fn current() -> Process {
    unsafe { scheduler() }.current_process().clone()
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
    inner: Arc<RwLock<ProcessData>>,
}

impl Process {
    pub fn new(address_space: AddressSpace) -> Self {
        let data = ProcessData::new(address_space);
        Self {
            id: ProcessId::new(),
            inner: Arc::new(RwLock::new(data)),
        }
    }

    pub fn process_id(&self) -> &ProcessId {
        &self.id
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

    pub fn address_space_mut(&mut self) -> &mut AddressSpace {
        &mut self.address_space
    }
}
