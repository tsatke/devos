use alloc::collections::VecDeque;
use core::mem::swap;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering::Relaxed;

use conquer_once::spin::OnceCell;
use crossbeam_queue::SegQueue;
use x86_64::instructions::{hlt, interrupts};

use crate::arch::switch::switch;
use crate::process::task::{Finished, Ready, Running, Task};
use crate::process::{process_tree, spawn_task_in_current_process, Process, ProcessId};
use crate::serial_println;

static mut SCHEDULER: Option<Scheduler> = None;
static FINISHED_TASKS: OnceCell<SegQueue<Task<Finished>>> = OnceCell::uninit();
static NEW_TASKS: OnceCell<SegQueue<Task<Ready>>> = OnceCell::uninit();

fn finished_tasks() -> &'static SegQueue<Task<Finished>> {
    FINISHED_TASKS
        .try_get()
        .expect("finished task queue not initialized")
}

fn new_tasks() -> &'static SegQueue<Task<Ready>> {
    NEW_TASKS.try_get().expect("new task queue not initialized")
}

pub fn init(kernel_task: Task<Running>) {
    unsafe { SCHEDULER = Some(Scheduler::new(kernel_task)) };
    FINISHED_TASKS.init_once(SegQueue::new);
    NEW_TASKS.init_once(SegQueue::new);

    // now that the finish queue is initialized, we can spawn the cleanup task
    spawn_task_in_current_process("cleanup_finished_tasks", cleanup_finished_tasks);
}

pub(crate) fn spawn(task: Task<Ready>) {
    new_tasks().push(task)
}

extern "C" fn cleanup_finished_tasks() {
    let queue = finished_tasks();
    loop {
        match queue.pop() {
            Some(task) => free_task(task),
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
    current_task: Task<Running>,
    current_task_should_exit: AtomicBool,
    ready: VecDeque<Task<Ready>>,
    _dummy_last_stack_ptr: usize,
}

impl Scheduler {
    /// Creates a new scheduler.
    ///
    /// # Safety
    /// Calling this more than once may result in UB due to aliasing
    /// of memory areas, tasks and processes.
    unsafe fn new(kernel_task: Task<Running>) -> Self {
        Self {
            current_task: kernel_task,
            current_task_should_exit: AtomicBool::new(false),
            ready: VecDeque::new(),
            _dummy_last_stack_ptr: 0,
        }
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

    pub fn current_pid(&self) -> &ProcessId {
        self.current_process().process_id()
    }

    pub fn current_process(&self) -> &Process {
        self.current_task.process()
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
            finished_tasks().push(task);
            &mut self._dummy_last_stack_ptr
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
}

fn free_task(task: Task<Finished>) {
    serial_println!(
        "freeing task {} ({}) in process {} ({})",
        task.task_id(),
        task.name(),
        task.process().process_id(),
        task.process().name()
    );

    // TODO: unwind

    // TODO: deallocate stack

    let mut process_tree = process_tree().write();
    let pid = *task.process().process_id();
    process_tree.remove_task(&pid, task.task_id());
    drop(task);

    if !process_tree.has_tasks(&pid) {
        let process = match process_tree.remove_process(&pid) {
            None => {
                panic!(
                    "tried to free process {}, but process doesn't exist in the process tree",
                    pid
                );
            }
            Some(p) => p,
        };

        // TODO: deallocate address space

        // TODO: close file descriptors

        drop(process);
    }
}
