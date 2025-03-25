use crate::mcore::mtask::task::Task;
use alloc::boxed::Box;

#[derive(Debug)]
pub struct Scheduler {
    current_task: Box<Task>,
}

impl Scheduler {
    pub fn new_cpu_local() -> Self {
        let current_task = unsafe { Task::create_current() }.into();
        Self { current_task }
    }

    pub fn current_task(&self) -> &Task {
        &self.current_task
    }
}
