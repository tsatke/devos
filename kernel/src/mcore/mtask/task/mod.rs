use crate::mcore::mtask::process::Process;
use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::sync::{Arc, Weak};
use core::pin::Pin;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering::Relaxed;
pub use id::*;
pub use stack::*;
pub use state::*;

mod id;
mod stack;
mod state;

#[derive(Debug)]
pub struct Task {
    /// The unique identifier of the task.
    tid: TaskId,
    /// The name of the task, not necessarily unique.
    name: String,
    /// The parent process that this task belongs to.
    /// If upon rescheduling, the parent process is not alive, the task will be terminated.
    process: Weak<Process>,
    /// Whether this task should be terminated upon the next reschedule.
    /// This can be set at any point.
    should_terminate: AtomicBool,
    /// The stack pointer of the task at the time of the last context switch.
    /// If this task is currently running, then this value is not the current stack pointer.
    /// This must be set during the context switch.
    last_stack_ptr: Pin<Box<usize>>,
    state: State,
    stack: Option<Stack>,
}

impl Task {
    /// Creates a Task struct for the current state of the CPU.
    /// The task is inactive, and its values must be set by the scheduler
    /// first.
    ///
    /// The resulting task will belong to the root process.
    ///
    /// # Safety
    /// The caller must ensure that this is only called once per core.
    pub unsafe fn create_current() -> Self {
        let tid = TaskId::new();
        let name = format!("task-{}", tid);
        let process = Arc::downgrade(Process::root());
        let should_terminate = AtomicBool::new(false);
        let last_stack_ptr = Box::pin(0);
        let state = State::Running;
        let stack = None;
        Self {
            tid,
            name,
            process,
            should_terminate,
            last_stack_ptr,
            state,
            stack,
        }
    }

    pub fn id(&self) -> TaskId {
        self.tid
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn process(&self) -> Option<Arc<Process>> {
        self.process.upgrade()
    }

    pub fn should_terminate(&self) -> bool {
        self.should_terminate.load(Relaxed)
    }

    pub fn set_should_terminate(&self, should_terminate: bool) {
        self.should_terminate.store(should_terminate, Relaxed);
    }

    pub fn state(&self) -> State {
        self.state
    }
}
