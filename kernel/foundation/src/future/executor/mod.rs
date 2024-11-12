use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use core::future::Future;
use core::hint::spin_loop;
use core::sync::atomic::Ordering::{Acquire, SeqCst};
use core::sync::atomic::{AtomicBool, AtomicUsize};
use core::task::{Context, Poll};
use crossbeam::queue::SegQueue;
use futures::channel::oneshot;
use spin::Mutex;
use task::{JoinHandle, Task, TaskId};

pub use single::block_on;

mod single;
mod task;

pub struct Executor {
    ready_queue: Arc<SegQueue<TaskId>>,
    ready_tasks: Mutex<BTreeMap<TaskId, Task>>,
    active_tasks: Arc<AtomicUsize>,
}

impl Executor {
    pub fn new() -> Self {
        Self {
            ready_queue: Arc::new(SegQueue::new()),
            ready_tasks: Mutex::new(BTreeMap::new()),
            active_tasks: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn spawn<F, T>(&self, future: F) -> JoinHandle<T>
    where
        F: Future<Output=T> + Send + Sync + 'static,
        T: Send + Sync + 'static,
    {
        let (tx, rx) = oneshot::channel();
        let should_cancel = Arc::new(AtomicBool::new(false));
        let handle = JoinHandle::new(rx, should_cancel.clone());

        let wrapper = async move {
            let _ = tx.send(future.await); // we don't care if the receiver was dropped
        };
        let wrapper = Box::pin(wrapper);

        let task = Task::new(self.ready_queue.clone(), wrapper, should_cancel, self.active_tasks.clone());
        let task_id = task.id();

        self.ready_tasks.lock().insert(task_id, task);
        self.active_tasks.fetch_add(1, SeqCst);
        self.ready_queue.push(task_id);

        handle
    }

    /// Execute a single task from the currently active
    /// ones, or do nothing if no task is available.
    ///
    /// When using this executor, call this in a loop
    /// to perform any work. This can be called
    /// simultaneously from multiple threads.
    pub fn execute_task(&self) -> ExecuteResult {
        let mut task = loop {
            let Some(next_task_id) = self.ready_queue.pop() else {
                return ExecuteResult::Idled;
            };

            // If we can't find a task, that means that the task is currently
            // being executed on another thread. We'll continue with the next one.
            let Some(task) = self.ready_tasks.lock().remove(&next_task_id) else {
                continue;
            };

            if task.should_cancel() {
                // Dropping the task will also drop the sender that sends the
                // result to the JoinHandler. This will cause the JoinHandler
                // to return None when polled.
                drop(task);
                continue;
            }

            break task;
        };

        // The task is now no longer in the active task list, but we own it here,
        // and we will execute it now. Afterward, we have to put the task back.

        let waker = task.waker().clone();

        let mut context = Context::from_waker(&waker);

        match task.future().poll(&mut context) {
            Poll::Ready(()) => {
                // task is done, no need to re-insert the task
            }
            Poll::Pending => {
                // task is not done yet, we need to re-insert the task
                let previous_task_with_same_id = self.ready_tasks.lock().insert(task.id(), task);
                debug_assert!(
                    previous_task_with_same_id.is_none(),
                    "task id must be unique"
                );
            }
        };

        ExecuteResult::Worked
    }

    pub fn run_active_tasks_to_completion(&self) {
        while self.active_tasks() > 0 {
            match self.execute_task() {
                ExecuteResult::Idled => spin_loop(),
                ExecuteResult::Worked => {}
            }
        }
    }

    pub fn active_tasks(&self) -> usize {
        self.active_tasks.load(Acquire)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ExecuteResult {
    Worked,
    Idled,
}
