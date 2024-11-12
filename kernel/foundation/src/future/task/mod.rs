use alloc::boxed::Box;
use alloc::sync::Arc;
use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering::Relaxed;
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

pub struct Task {
    id: TaskId,
    waker: Waker,
    future: Pin<Box<dyn Future<Output = ()> + Send + Sync + 'static>>,
}

impl Task {
    pub(crate) fn new(
        ready_queue: Arc<SegQueue<TaskId>>,
        future: Pin<Box<impl Future<Output = ()> + Send + Sync + 'static>>,
    ) -> Self {
        let id = TaskId::new();
        let waker = TaskWaker::new(id, ready_queue);
        Self { id, waker, future }
    }

    pub fn id(&self) -> TaskId {
        self.id
    }

    pub fn waker(&self) -> &Waker {
        &self.waker
    }

    pub fn future(&mut self) -> Pin<&mut (dyn Future<Output = ()> + Send + Sync)> {
        self.future.as_mut()
    }
}
