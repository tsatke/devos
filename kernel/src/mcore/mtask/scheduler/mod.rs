use crate::mcore::mtask::scheduler::global::GlobalTaskQueue;
use crate::mcore::mtask::scheduler::switch::switch_impl;
use crate::mcore::mtask::task::Task;
use alloc::boxed::Box;
use core::cell::UnsafeCell;
use core::mem::swap;
use core::pin::Pin;
use x86_64::instructions::interrupts;

pub mod global;
mod switch;

#[derive(Debug)]
pub struct Scheduler {
    /// The task that is currently executing in this scheduler.
    current_task: Pin<Box<Task>>,
    /// A dummy location that is a placeholder for the switch code to write the old stack
    /// pointer to if the old task is terminated.
    dummy_old_stack_ptr: UnsafeCell<usize>,
}

impl Scheduler {
    #[must_use]
    pub fn new_cpu_local() -> Self {
        let current_task = Box::pin(unsafe { Task::create_current() });
        Self {
            current_task,
            dummy_old_stack_ptr: UnsafeCell::new(0),
        }
    }

    /// # Safety
    /// Trivially unsafe. If you don't know why, please don't call this function.
    pub unsafe fn reschedule(&mut self) {
        interrupts::disable();

        let (next_task, cr3_value) = {
            let Some(next_task) = self.next_task() else {
                interrupts::enable();
                return;
            };

            let cr3_value = next_task.process().address_space().cr3_value();
            (next_task, cr3_value)
        };

        let mut old_task = self.swap_current_task(next_task);
        let old_stack_ptr = if old_task.should_terminate() {
            self.dummy_old_stack_ptr.get()
        } else {
            old_task.last_stack_ptr() as *mut usize
        };
        if !old_task.should_terminate() {
            GlobalTaskQueue::enqueue(old_task);
        }

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

    #[must_use]
    pub fn current_task(&self) -> &Task {
        &self.current_task
    }

    fn swap_current_task(&mut self, next_task: Pin<Box<Task>>) -> Pin<Box<Task>> {
        let mut next_task = next_task;
        swap(&mut self.current_task, &mut next_task);
        next_task
    }

    #[allow(clippy::unused_self)]
    fn next_task(&self) -> Option<Pin<Box<Task>>> {
        GlobalTaskQueue::dequeue()
    }
}
