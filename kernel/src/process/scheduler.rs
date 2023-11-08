use alloc::collections::VecDeque;
use core::mem::swap;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering::Relaxed;

use conquer_once::spin::OnceCell;
use crossbeam_queue::{ArrayQueue, SegQueue};
use x86_64::instructions::{hlt, interrupts};

use crate::arch::switch::switch;
use crate::process::task::{Finished, Ready, Running, Task};
use crate::process::{spawn_task_in_current_process, Process, ProcessId, ProcessTree};
use crate::serial_println;

static mut SCHEDULER: Option<Scheduler> = None;
static FINISHED_TASKS: OnceCell<ArrayQueue<Task<Finished>>> = OnceCell::uninit();
static NEW_TASKS: OnceCell<SegQueue<Task<Ready>>> = OnceCell::uninit();

fn finished_tasks() -> &'static ArrayQueue<Task<Finished>> {
    FINISHED_TASKS
        .try_get()
        .expect("finished task queue not initialized")
}

fn new_tasks() -> &'static SegQueue<Task<Ready>> {
    NEW_TASKS.try_get().expect("new task queue not initialized")
}

pub fn init(root_process: Process) {
    unsafe { SCHEDULER = Some(Scheduler::new(root_process)) };
    FINISHED_TASKS.init_once(|| ArrayQueue::new(10));
    NEW_TASKS.init_once(|| SegQueue::new());

    // now that the scheduler is initialized, we can spawn the cleanup task
    spawn_task_in_current_process("cleanup_finished_tasks", cleanup_finished_tasks);
}

pub(crate) fn spawn(task: Task<Ready>) {
    new_tasks().push(task)
}

extern "C" fn cleanup_finished_tasks() {
    let queue = finished_tasks();
    loop {
        match queue.pop() {
            Some(task) => {
                unsafe { scheduler_mut().free_task(task) };
            }
            None => {
                hlt();
                continue;
            }
        }
    }
}

pub(in crate::process) unsafe fn scheduler() -> &'static Scheduler {
    SCHEDULER.as_ref().unwrap()
}

pub(in crate::process) unsafe fn scheduler_mut() -> &'static mut Scheduler {
    SCHEDULER.as_mut().unwrap()
}

/// # Safety
/// This can only be called after [`init`] has been called.
/// This may or may not return, make sure to use it in a way that can handle both cases.
pub(crate) unsafe fn reschedule() {
    unsafe { scheduler_mut().reschedule() }
}

pub(crate) unsafe fn exit_current_task() -> ! {
    unsafe { scheduler().exit_current_task() }
}

pub struct Scheduler {
    process_tree: ProcessTree,
    current_task: Task<Running>,
    current_task_should_exit: AtomicBool,
    ready: VecDeque<Task<Ready>>,
    finished: VecDeque<Task<Finished>>,
}

impl Scheduler {
    /// Creates a new scheduler.
    ///
    /// # Safety
    /// Calling this more than once may result in UB due to aliasing
    /// of memory areas, tasks and processes.
    unsafe fn new(root_process: Process) -> Self {
        let current_task = Task::kernel_task(root_process.clone());

        Self {
            process_tree: ProcessTree::new(root_process, current_task.task_id()),
            current_task,
            current_task_should_exit: AtomicBool::new(false),
            ready: VecDeque::new(),
            finished: VecDeque::new(),
        }
    }

    pub(in crate::process) fn process_tree_mut(&mut self) -> &mut ProcessTree {
        &mut self.process_tree
    }

    pub fn spawn(&mut self, task: Task<Ready>) {
        #[cfg(debug_assertions)]
        {
            self.process_tree
                .process_by_id(task.process().process_id())
                .expect(
                    "the process of the task must be in the process tree before spawning the task",
                );
        }
        self.process_tree
            .add_task(task.process().process_id(), task.task_id());
        self.ready.push_back(task);
    }

    pub fn exit_current_task(&self) -> ! {
        self.current_task_should_exit.store(true, Relaxed);
        loop {
            hlt();
        }
    }

    pub fn current_task(&self) -> &Task<Running> {
        &self.current_task
    }

    pub fn current_pid(&self) -> ProcessId {
        *self.current_task().process().process_id()
    }

    pub fn current_process(&self) -> &Process {
        self.process_tree
            .process_by_id(&self.current_pid())
            .expect("there must be a current process")
    }

    /// Reschedules to another task.
    ///
    /// This may or may not return.
    ///
    /// # Safety
    /// This is highly unsafe, since we do a lot of things that are not safe, including but
    /// not limited to:
    /// * switching the address space (trivially unsafe)
    /// * switching rings
    /// * switching stacks
    /// * modifying the instruction pointer
    ///
    /// Only call this on timer interrupts and if you know what you're doing.
    pub unsafe fn reschedule(&mut self) {
        /*
        IMPORTANT!!!
        WE CAN NOT ACQUIRE ANY LOCKS!!!
        This will cause deadlocks and/or instability!

        Imagine the lock being held by the current task, so we are unable
        to acquire the lock in here. This means, we won't be able to switch
        to another task, and the current task will never release the lock.
        */

        // move new tasks to the ready queue
        while let Some(task) = new_tasks().pop() {
            self.ready.push_back(task);
        }

        // move finished tasks to the finished queue
        while let Some(task) = self.finished.pop_front() {
            match finished_tasks().push(task) {
                Ok(_) => {}
                Err(t) => self.finished.push_front(t),
            };
        }

        let task = match self.ready.pop_front() {
            None => {
                return;
            }
            Some(t) => t,
        };

        let cr3_value = task.process().cr3_value();

        /*
        @dev please note that from here on, you have to enable interrupts manually if you wish to exit early
        and that this will not be done for you except if the method ends in the actual task switch
        */
        interrupts::disable(); // will be enabled again during task switch (in assembly)

        // swap out the task from the queue and the current task
        let mut task = task.into_running();
        swap(&mut self.current_task, &mut task);

        let should_exit = self.current_task_should_exit.swap(false, Relaxed);
        let old_stack_ptr_ref = if should_exit {
            let task = task.into_finished();
            self.finished.push_back(task);
            self.finished.back_mut().unwrap().last_stack_ptr_mut()
        } else {
            let task = task.into_ready();
            self.ready.push_back(task);
            self.ready.back_mut().unwrap().last_stack_ptr_mut()
        };

        // hope that this works
        let old_stack_ptr = {
            // Safety: this is not really safe and may break at any time.
            // Feel free to find a better (safe) way.
            // TODO: Can we maybe use [`Pin`] here?
            //
            // We get the pointer to the `last_stack_ptr` field of the last element in the ready
            // queue - which we just pushed - and pass that into the switch, so that the assembly
            // in there can write the most recent stack pointer to that location.
            old_stack_ptr_ref as *mut usize
        };

        let new_stack_ptr = *self.current_task.last_stack_ptr() as *const u8;

        unsafe { switch(old_stack_ptr, new_stack_ptr, cr3_value) }
    }

    fn free_task(&mut self, task: Task<Finished>) {
        serial_println!(
            "freeing task {} ({}) in process {} ({})",
            task.task_id(),
            task.name(),
            task.process().process_id(),
            task.process().name()
        );

        // TODO: unwind

        // TODO: deallocate stack

        let pid = task.process().process_id();
        self.process_tree.remove_task(pid, task.task_id());
        if !self.process_tree.has_tasks(pid) {
            self.free_process(pid);
        }

        drop(task);
    }

    fn free_process(&mut self, process_id: &ProcessId) {
        if self.process_tree.has_tasks(process_id) {
            panic!(
                "attempted to free process {}, but it still has tasks",
                process_id
            );
        }

        let process = match self.process_tree.remove_process(process_id) {
            None => {
                panic!(
                    "tried to free process {}, but process doesn't exist in the process tree",
                    process_id
                );
            }
            Some(v) => v,
        };

        // TODO: deallocate address space

        // TODO: close file descriptors

        drop(process);
    }
}
