use crate::future::task::JoinHandle;
use crate::future::task::{Task, TaskId};
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use core::future::Future;
use core::hint::spin_loop;
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering::SeqCst;
use core::task::{Context, Poll};
use crossbeam::queue::SegQueue;
use spin::Mutex;

pub struct Executor {
    ready_queue: Arc<SegQueue<TaskId>>,
    ready_tasks: Mutex<BTreeMap<TaskId, Task>>,
    active_tasks: AtomicUsize,
}

impl Executor {
    pub fn new() -> Self {
        Self {
            ready_queue: Arc::new(SegQueue::new()),
            ready_tasks: Mutex::new(BTreeMap::new()),
            active_tasks: AtomicUsize::new(0),
        }
    }

    pub fn spawn<F, T>(&self, future: F) -> Result<JoinHandle<T>, ()>
    where
        F: Future<Output = T> + Send + Sync + 'static,
        T: Send + Sync + 'static,
    {
        let handle = JoinHandle::default();

        let out = handle.result();
        let wrapper = async move {
            let _ = out.lock().insert(future.await);
        };
        let wrapper = Box::pin(wrapper);

        let task = Task::new(self.ready_queue.clone(), wrapper);
        let task_id = task.id();

        self.ready_tasks.lock().insert(task_id, task);
        self.active_tasks.fetch_add(1, SeqCst);
        self.ready_queue.push(task_id);

        Ok(handle)
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
            if let Some(task) = self.ready_tasks.lock().remove(&next_task_id) {
                break task;
            }
        };

        // The task is now no longer in the active task list, but we own it here,
        // and we will execute it now. Afterward, we have to put the task back.

        let waker = task.waker().clone();

        let mut context = Context::from_waker(&waker);

        match task.future().poll(&mut context) {
            Poll::Ready(()) => {
                // task is done, no need to re-insert the task
                self.active_tasks.fetch_sub(1, SeqCst);
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
        self.ready_tasks.lock().len()
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ExecuteResult {
    Worked,
    Idled,
}
