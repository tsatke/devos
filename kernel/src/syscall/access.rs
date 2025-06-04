use crate::mcore::context::ExecutionContext;
use crate::mcore::mtask::process::Process;
use crate::mcore::mtask::task::Task;
use crate::vfs::vfs;
use alloc::sync::Arc;
use kernel_syscall::fcntl::{ProcessAccess, TaskAccess, VfsAccess};
use kernel_vfs::Vfs;
use spin::rwlock::RwLock;

pub struct KernelAccess<'a> {
    task: KernelTask<'a>,
    process: KernelProcess,
}

impl<'a> KernelAccess<'a> {
    pub fn new() -> Self {
        let task = ExecutionContext::load().current_task();
        let process = task.process().clone();
        let task = task.into();
        let process = process.into();

        KernelAccess { task, process }
    }
}

impl ProcessAccess for KernelAccess<'_> {
    type Process = KernelProcess;

    fn current_process(&self) -> &Self::Process {
        &self.process
    }
}

impl<'a> TaskAccess for KernelAccess<'a> {
    type Task = KernelTask<'a>;

    fn current_task(&self) -> &Self::Task {
        &self.task
    }
}

pub struct KernelTask<'a> {
    _task: &'a Task,
}

impl<'a> From<&'a Task> for KernelTask<'a> {
    fn from(task: &'a Task) -> Self {
        KernelTask { _task: task }
    }
}

impl kernel_syscall::fcntl::Task for KernelTask<'_> {}

pub struct KernelProcess {
    process: Arc<Process>,
}

impl From<Arc<Process>> for KernelProcess {
    fn from(process: Arc<Process>) -> Self {
        KernelProcess { process }
    }
}

impl kernel_syscall::fcntl::Process for KernelProcess {
    fn current_dir(&self) -> &RwLock<kernel_vfs::path::AbsoluteOwnedPath> {
        self.process.current_working_directory()
    }
}

impl VfsAccess for KernelAccess<'_> {
    fn vfs(&self) -> &RwLock<Vfs> {
        vfs()
    }
}
