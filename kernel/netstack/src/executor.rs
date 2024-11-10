use alloc::boxed::Box;
use alloc::sync::Arc;
use core::future::Future;
use core::hint::spin_loop;
use core::pin::Pin;
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering::SeqCst;
use core::task::{Context, Poll};
use crossbeam::queue::SegQueue;
use futures::task::{waker_ref, ArcWake};
use spin::Mutex;

type PendingTaskQueue = SegQueue<Arc<Task>>;

pub struct Executor {
    ready_queue: Arc<PendingTaskQueue>,
    active_tasks: AtomicUsize,
}

impl Executor {
    pub fn new() -> Self {
        Self {
            ready_queue: Arc::new(SegQueue::new()),
            active_tasks: AtomicUsize::new(0),
        }
    }

    pub fn spawn<F, T>(&self, future: F) -> Result<JoinHandle<T>, ()>
    where
        F: Future<Output = T> + Send + Sync + 'static,
        T: Send + Sync + 'static,
    {
        let handle = JoinHandle::default();

        let out = handle.result.clone();
        let wrapper = async move {
            let _ = out.lock().insert(future.await);
        };
        let wrapper = Box::pin(wrapper);

        let task = Task::new(self.ready_queue.clone(), wrapper);

        self.active_tasks.fetch_add(1, SeqCst);
        self.ready_queue.push(Arc::new(task));

        Ok(handle)
    }

    /// Execute a single task from the currently active
    /// ones, or do nothing if no task is available.
    ///
    /// When using this executor, call this in a loop
    /// to perform any work. This can be called
    /// simultaneously from multiple threads.
    pub fn execute_task(&self) -> ExecuteResult {
        let Some(next_task) = self.ready_queue.pop() else {
            return ExecuteResult::Idled;
        };

        let mut future_slot = next_task.future.lock();
        if let Some(mut future) = future_slot.take() {
            let waker = waker_ref(&next_task);
            let mut cx = Context::from_waker(&waker);
            match future.as_mut().poll(&mut cx) {
                Poll::Ready(_) => {
                    self.active_tasks.fetch_sub(1, SeqCst);
                }
                Poll::Pending => {
                    *future_slot = Some(future);
                }
            }
        }

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
        self.active_tasks.load(SeqCst)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ExecuteResult {
    Worked,
    Idled,
}

pub struct Task {
    ready_queue: Arc<PendingTaskQueue>,
    future: Mutex<Option<Pin<Box<dyn Future<Output = ()> + Send + Sync + 'static>>>>,
}

impl Task {
    pub fn new(
        ready_queue: Arc<PendingTaskQueue>,
        future: Pin<Box<impl Future<Output = ()> + Send + Sync + 'static>>,
    ) -> Self {
        Self {
            ready_queue,
            future: Mutex::new(Some(future)),
        }
    }
}

impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        let task = arc_self.clone();
        let queue = task.ready_queue.clone();
        queue.push(task);
    }
}

pub struct JoinHandle<T> {
    result: Arc<Mutex<Option<T>>>,
}

impl<T> Default for JoinHandle<T> {
    fn default() -> Self {
        Self {
            result: Arc::new(Mutex::new(None)),
        }
    }
}

impl<T> JoinHandle<T> {
    pub fn get_result(&self) -> Option<T> {
        self.result.lock().take()
    }
}
