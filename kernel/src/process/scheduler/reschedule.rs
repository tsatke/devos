use alloc::boxed::Box;
use core::mem::swap;
use core::pin::Pin;
use core::sync::atomic::Ordering::{Relaxed, Release};

use x86_64::instructions::interrupts;

use crate::arch::switch::switch;
use crate::process::scheduler::{finished_threads, new_threads};
use crate::process::thread::{State, Thread};
use crate::process::{Priority, Scheduler, IN_RESCHEDULE};

impl Scheduler {
    /// Reschedules to another thread.
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
        if IN_RESCHEDULE
            .compare_exchange(false, true, Release, Relaxed)
            .is_err()
        {
            return;
        }

        /*
        @dev please note that from here on, you have to enable interrupts manually if you wish to exit early
        and that this will not be done for you except if the method ends in the actual task switch
        */
        interrupts::disable(); // will be enabled again during task switch (in assembly)

        /*
        IMPORTANT!!!
        WE CAN NOT ACQUIRE ANY LOCKS!!! (allocating memory acquires locks)
        This will cause deadlocks and/or instability!

        Imagine the lock being held by the current thread, so we are unable
        to acquire the lock in here. This means, we won't be able to switch
        to another thread, and the current thread will never release the lock.
        */

        // move new threads from queue into scheduler
        self.take_new_threads();

        // compute the next thread
        let next_thread = self.next_thread();

        // swap the current thread with the next thread and get the priority for the old thread,
        // because it might have changed (or better: we still need to change it)
        let (priority, mut old_thread) = self.swap_current_thread(next_thread);

        let should_exit = self.current_thread_should_exit.swap(false, Relaxed);
        let old_stack_ptr = if should_exit {
            old_thread.set_state(State::Finished);
            finished_threads().enqueue(Box::into_pin(old_thread));
            &mut self._dummy_last_stack_ptr as *mut usize
        } else {
            old_thread.set_state(State::Ready);
            let last_stack_ptr = old_thread.last_stack_ptr_mut().as_mut().get_mut() as *mut usize;
            self.ready[priority].enqueue(Box::into_pin(old_thread));
            last_stack_ptr
        };

        let new_stack_ptr = *self.current_thread.last_stack_ptr().as_ref() as *const u8;
        let cr3_value = self.current_thread.process().cr3_value();

        IN_RESCHEDULE.store(false, Relaxed);

        unsafe { switch(old_stack_ptr, new_stack_ptr, cr3_value) }
    }

    fn swap_current_thread(&mut self, next_thread: Box<Thread>) -> (Priority, Box<Thread>) {
        let mut next_thread = next_thread;
        next_thread.set_state(State::Running);
        assert_eq!(self.current_thread.state(), next_thread.state());
        let new_priority_for_old_task = self
            .current_thread_prio
            .swap(next_thread.priority(), Relaxed);

        swap(self.current_thread.as_mut(), &mut next_thread);
        (new_priority_for_old_task, next_thread)
    }

    fn next_thread(&mut self) -> Box<Thread> {
        let thread = {
            // this loop terminates because we must have at least the idle thread in a ready queue
            // (which is the old kernel task, that is in a hlt-loop)
            loop {
                if let Some(thread) = self.ready[self.strategy.next().unwrap()].dequeue() {
                    break Pin::into_inner(thread);
                }
            }
        };
        thread
    }

    fn take_new_threads(&mut self) {
        // We don't care about the err case, whether it is because the queue is empty,
        // in an inconsistent state or busy, we try again anyway. We don't want to way,
        // which is why we use `try_dequeue` instead of `dequeue`, since the latter
        // contains an implicit exponential backoff.
        while let Ok(thread) = new_threads().try_dequeue() {
            self.ready[thread.priority()].enqueue(thread);
        }
    }
}
