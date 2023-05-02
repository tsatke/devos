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

/// Reschedules to another task.
///
/// # Safety
/// This is highly unsafe, since we do a lot of things that are not safe, including but
/// not limited to:
/// * switching the address space (trivially unsafe)
/// * switching rings
/// * switching stacks
/// * modifying the instruction pointer
pub unsafe fn reschedule() {
    // TODO: implement
}
