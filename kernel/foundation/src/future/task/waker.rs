use crate::future::task::TaskId;
use alloc::sync::Arc;
use alloc::task::Wake;
use core::task::Waker;
use crossbeam::queue::SegQueue;

#[derive(Clone)]
pub struct TaskWaker {
    id: TaskId,
    ready_queue: Arc<SegQueue<TaskId>>,
}

impl TaskWaker {
    pub fn new(id: TaskId, ready_queue: Arc<SegQueue<TaskId>>) -> Waker {
        Waker::from(Arc::new(Self { id, ready_queue }))
    }
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref()
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.ready_queue.push(self.id)
    }
}
