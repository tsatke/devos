use alloc::boxed::Box;
use core::arch::asm;
use core::arch::x86_64::_fxsave;
use core::cell::UnsafeCell;
use core::mem::swap;
use core::pin::Pin;

use x86_64::instructions::interrupts;
use x86_64::registers::model_specific::FsBase;

use crate::mcore::mtask::scheduler::global::GlobalTaskQueue;
use crate::mcore::mtask::scheduler::switch::switch_impl;
use crate::mcore::mtask::task::Task;

pub mod global;
mod switch;

#[derive(Debug)]
pub struct Scheduler {
    /// The task that is currently executing in this scheduler.
    current_task: Pin<Box<Task>>,
    /// The task this scheduler last switched away from. We need this to
    /// eliminate the race condition between re-queueing a task and
    /// actually switching away from it.
    zombie_task: Option<Pin<Box<Task>>>,
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
            zombie_task: None,
            dummy_old_stack_ptr: UnsafeCell::new(0),
        }
    }

    /// # Safety
    /// Trivially unsafe. If you don't know why, please don't call this function.
    pub unsafe fn reschedule(&mut self) {
        // in theory, we could move this to the end of this function, but I'd rather not do this right now
        if let Some(zombie_task) = self.zombie_task.take()
            && !zombie_task.should_terminate()
        {
            GlobalTaskQueue::enqueue(zombie_task);
        }

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

        if let Some(mut guard) = old_task.fx_area().try_write()
            && let Some(fx_area) = guard.as_mut()
        {
            unsafe { asm!("clts") };
            unsafe {
                // Safety: Safe because we hold a mutable reference to the fx_area
                _fxsave(fx_area.start().as_mut_ptr::<u8>());
            }
        }

        if let Some(guard) = self.current_task.tls().try_read()
            && let Some(tls) = guard.as_ref()
        {
            FsBase::write(tls.start());
        }

        assert!(self.zombie_task.is_none());
        self.zombie_task = Some(old_task);

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
