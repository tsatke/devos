use crate::mem::AddressSpace;
use core::sync::atomic::AtomicU64;
use core::sync::atomic::Ordering::SeqCst;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct TaskId(u64);

impl !Default for TaskId {}

impl TaskId {
    pub fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        TaskId(COUNTER.fetch_add(1, SeqCst))
    }
}

pub struct Task {
    id: TaskId,
    address_space: AddressSpace,
}

impl Task {
    pub fn new(address_space: AddressSpace) -> Self {
        Self {
            id: TaskId::new(),
            address_space,
        }
    }

    pub fn id(&self) -> &TaskId {
        &self.id
    }

    pub fn address_space_mut(&mut self) -> &mut AddressSpace {
        &mut self.address_space
    }
}
