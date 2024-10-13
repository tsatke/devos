use core::ops::Index;
use core::sync::atomic::{AtomicUsize, Ordering};

use derive_more::Display;
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[repr(transparent)]
pub struct AtomicPriority {
    inner: AtomicUsize,
}

impl AtomicPriority {
    pub fn new(priority: Priority) -> Self {
        Self {
            inner: AtomicUsize::new(priority.into()),
        }
    }

    pub fn store(&self, priority: Priority, ordering: Ordering) {
        self.inner.store(priority.into(), ordering)
    }

    pub fn swap(&self, priority: Priority, ordering: Ordering) -> Priority {
        Priority::try_from_primitive(self.inner.swap(priority.into(), ordering)).unwrap()
    }
}

#[derive(Display, Debug, Copy, Clone, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(usize)]
pub enum Priority {
    Low = 0,
    Normal,
    High,
    Realtime,
}

pub struct Queues<T> {
    queues: [T; 4],
}

impl<T> Queues<T> {
    pub fn new(low: T, normal: T, high: T, realtime: T) -> Self {
        Self {
            queues: [low, normal, high, realtime],
        }
    }
}

impl<T> Index<Priority> for Queues<T> {
    type Output = T;

    fn index(&self, index: Priority) -> &Self::Output {
        &self.queues[index as usize]
    }
}
