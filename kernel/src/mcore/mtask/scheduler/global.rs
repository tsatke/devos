use crate::mcore::mtask::task::{Task, TaskQueue};
use alloc::boxed::Box;
use conquer_once::spin::OnceCell;
use core::pin::Pin;

static GLOBAL_QUEUE: OnceCell<TaskQueue> = OnceCell::uninit();

fn global_queue() -> &'static TaskQueue {
    GLOBAL_QUEUE.get_or_init(TaskQueue::new)
}

pub struct GlobalTaskQueue;

impl GlobalTaskQueue {
    pub fn enqueue(task: Pin<Box<Task>>) {
        global_queue().enqueue(task);
    }

    #[must_use]
    pub fn dequeue() -> Option<Pin<Box<Task>>> {
        global_queue().dequeue()
    }
}
