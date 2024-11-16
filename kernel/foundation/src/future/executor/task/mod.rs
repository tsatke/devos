use alloc::boxed::Box;
use alloc::sync::Arc;
use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::Ordering::{Acquire, Relaxed};
use core::sync::atomic::{AtomicBool, AtomicUsize};
use core::task::Waker;
use crossbeam::queue::SegQueue;
use waker::TaskWaker;

pub use join::*;

mod join;
mod waker;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TaskId(usize);

impl TaskId {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        Self(NEXT.fetch_add(1, Relaxed))
    }
}

pub struct Task<'a> {
    id: TaskId,
    waker: Waker,
    future: Pin<Box<dyn Future<Output=()> + Send + Sync + 'a>>,
    should_cancel: Arc<AtomicBool>,
    active_tasks: Arc<AtomicUsize>,
}

impl<'a> Task<'a> {
    pub(crate) fn new(
        ready_queue: Arc<SegQueue<TaskId>>,
        future: Pin<Box<impl Future<Output=()> + Send + Sync + 'a>>,
        should_cancel: Arc<AtomicBool>,
        active_tasks: Arc<AtomicUsize>,
    ) -> Self {
        let id = TaskId::new();
        let waker = TaskWaker::new_waker(id, ready_queue);
        Self { id, waker, future, should_cancel, active_tasks }
    }

    pub fn should_cancel(&self) -> bool {
        self.should_cancel.load(Acquire)
    }

    pub fn id(&self) -> TaskId {
        self.id
    }

    pub fn waker(&self) -> &Waker {
        &self.waker
    }

    pub fn future(&mut self) -> Pin<&mut (dyn Future<Output=()> + Send + Sync)> {
        self.future.as_mut()
    }
}

impl Drop for Task<'_> {
    fn drop(&mut self) {
        self.active_tasks.fetch_sub(1, Acquire);
    }
}