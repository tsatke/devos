use crate::mcore::mtask::scheduler::switch::switch_impl;
use crate::mcore::mtask::task::{Task, TaskQueue};
use alloc::boxed::Box;
use core::mem::swap;
use core::pin::Pin;
use x86_64::instructions::{hlt, interrupts};

mod switch;

#[derive(Debug)]
pub struct Scheduler {
    current_task: Pin<Box<Task>>,
    local_queue: TaskQueue,
}

impl Scheduler {
    #[must_use]
    pub fn new_cpu_local() -> Self {
        let current_task = Box::pin(unsafe { Task::create_current() });
        let local_queue = TaskQueue::new();
        Self {
            current_task,
            local_queue,
        }
    }

    pub fn enqueue(&self, task: Task) {
        let task = Box::pin(task);
        self.local_queue.enqueue(task);
    }

    /// # Safety
    /// Trivially unsafe. If you don't know why, please don't call this function.
    pub unsafe fn reschedule(&mut self) {
        interrupts::disable();

        let (next_task, cr3_value) = {
            let Some(next_task) = self.next_task() else {
                // this is our idle task implementation
                interrupts::enable();
                hlt();
                return;
            };

            let cr3_value = next_task.process().address_space().cr3_value();
            (next_task, cr3_value)
        };

        let mut old_task = self.swap_current_task(next_task);
        let old_stack_ptr = old_task.last_stack_ptr() as *mut usize;
        self.local_queue.enqueue(old_task);

        unsafe {
            Self::switch(
                &mut *old_stack_ptr, // yay, UB (but how else are we going to do this?)
                *self.current_task.last_stack_ptr(),
                cr3_value,
            );
        }
    }

    unsafe fn switch(old_stack_ptr: &mut usize, new_stack_ptr: usize, new_cr3_value: usize) {
        unsafe {
            switch_impl(
                core::ptr::from_mut::<usize>(old_stack_ptr),
                new_stack_ptr as *const u8,
                new_cr3_value,
            );
        }
    }

    pub fn current_task(&self) -> &Task {
        &self.current_task
    }

    fn swap_current_task(&mut self, next_task: Pin<Box<Task>>) -> Pin<Box<Task>> {
        let mut next_task = next_task;
        swap(&mut self.current_task, &mut next_task);
        next_task
    }

    fn next_task(&self) -> Option<Pin<Box<Task>>> {
        if let Some(thread) = self.local_queue.dequeue() {
            return Some(thread);
        }

        // TODO: try global queue for more work

        None
    }
}
