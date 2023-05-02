use crate::process::task::Task;
use lazy_static::lazy_static;
use spin::{RwLock, RwLockReadGuard, RwLockWriteGuard};

pub mod syscall;
pub mod task;

lazy_static! {
    static ref CURRENT_TASK: RwLock<Option<Task>> = RwLock::new(None);
}

pub fn current_task<'a>() -> RwLockReadGuard<'a, Option<Task>> {
    CURRENT_TASK.read()
}

pub fn current_task_mut<'a>() -> RwLockWriteGuard<'a, Option<Task>> {
    CURRENT_TASK.write()
}

pub unsafe fn reschedule() {}
