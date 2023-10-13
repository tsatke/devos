use crate::arch::switch::switch;
use crate::process::task::{Finished, Ready, Running, Task, TaskState};
use crate::process::{Process, ProcessId, ProcessTree};
use crate::serial_println;
use alloc::collections::VecDeque;
use core::mem::{swap, MaybeUninit};
use x86_64::instructions::interrupts;

static mut SCHEDULER: MaybeUninit<Scheduler> = MaybeUninit::uninit();

pub fn init(root_process: Process) {
    unsafe { SCHEDULER.write(Scheduler::new(root_process)) };
}

/// # Safety
/// This can only be called after [`init`] has been called.
/// This may or may not return, make sure to use it in a way that can handle both cases.
pub(crate) unsafe fn reschedule() {
    unsafe {
        SCHEDULER
            .assume_init_mut() // safe because this function must only be called after init
            .reschedule()
    }
}

/// # Safety
/// This is unsafe because it may alias the scheduler.
/// Make sure that you are outside of a ['reschedule'] call
/// and that there does not exist a mutable reference to the scheduler.
pub(crate) unsafe fn spawn(task: Task<Ready>) {
    unsafe { SCHEDULER.assume_init_mut().spawn(task) }
}

pub(in crate::process) unsafe fn scheduler() -> &'static Scheduler {
    SCHEDULER.assume_init_ref()
}

pub struct Scheduler {
    process_tree: ProcessTree,
    current_task: Task<Running>,
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
            process_tree: ProcessTree::new(root_process),
            current_task,
            ready: VecDeque::new(),
            finished: VecDeque::new(),
        }
    }

    pub fn spawn(&mut self, task: Task<Ready>) {
        self.ready.push_back(task);
    }

    pub fn current_task(&self) -> &Task<Running> {
        &self.current_task
    }

    pub fn current_pid(&self) -> ProcessId {
        *self.current_task().process().process_id()
    }

    pub fn current_process(&self) -> &Process {
        self.process_tree
            .process(&self.current_pid())
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
        // @dev please note that you have to enable interrupts manually if you wish to exit early
        // and that this will not be done for you except if the method ends in the actual task switch

        interrupts::disable(); // will be enabled again during task switch (in assembly)

        while let Some(task) = self.finished.pop_front() {
            self.free_task(task);
        }

        let task = match self.ready.pop_front() {
            None => {
                interrupts::enable();
                return;
            }
            Some(t) => t,
        };

        // swap out the task from the queue and the current task
        let mut task = task.into_running();
        swap(&mut self.current_task, &mut task);
        let task = task.into_ready();
        self.ready.push_back(task);

        // hope that this works
        let old_stack_ptr = {
            // Safety: this is not really safe and may break at any time.
            // Feel free to find a better (safe) way.
            // TODO: Can we maybe use [`Pin`] here?
            //
            // We get the pointer to the `last_stack_ptr` field of the last element in the ready
            // queue - which we just pushed - and pass that into the switch, so that the assembly
            // in there can write the most recent stack pointer to that location.
            self.ready.back_mut().unwrap().last_stack_ptr_mut() as *mut usize
        };

        let new_stack_ptr = *self.current_task.last_stack_ptr() as *const u8;
        let t = unsafe { &*(new_stack_ptr as *const TaskState) };
        let new_ip = unsafe { (new_stack_ptr as *const u64).add(17) };

        serial_println!(
            r#"switching to task {}
new sp: {:0x?}
new ip: {:0x?}
new task state: {:#0x?}"#,
            self.current_task.task_id(),
            new_stack_ptr,
            (*new_ip) as *const u8,
            t,
        );

        unsafe { switch(old_stack_ptr, new_stack_ptr) }
    }

    fn free_task(&mut self, task: Task<Finished>) {
        // TODO: unwind

        // TODO: deallocate stack

        // TODO: update the process tree

        drop(task);
    }
}
