use alloc::collections::VecDeque;
use core::mem::swap;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering::Relaxed;

use conquer_once::spin::OnceCell;
use crossbeam_queue::SegQueue;
use x86_64::instructions::{hlt, interrupts};

use crate::arch::switch::switch;
use crate::process::{Process, process_tree, spawn_thread_in_current_process};
use crate::process::attributes::ProcessId;
use crate::process::thread::{Finished, Ready, Running, Thread};
use crate::serial_println;

static mut SCHEDULER: Option<Scheduler> = None;
static FINISHED_THREADS: OnceCell<SegQueue<Thread<Finished>>> = OnceCell::uninit();
static NEW_THREADS: OnceCell<SegQueue<Thread<Ready>>> = OnceCell::uninit();

fn finished_threads() -> &'static SegQueue<Thread<Finished>> {
    FINISHED_THREADS
        .try_get()
        .expect("finished thread queue not initialized")
}

fn new_threads() -> &'static SegQueue<Thread<Ready>> {
    NEW_THREADS
        .try_get()
        .expect("new thread queue not initialized")
}

pub fn init(kernel_thread: Thread<Running>) {
    unsafe { SCHEDULER = Some(Scheduler::new(kernel_thread)) };
    FINISHED_THREADS.init_once(SegQueue::new);
    NEW_THREADS.init_once(SegQueue::new);

    // now that the finish queue is initialized, we can spawn the cleanup thread
    spawn_thread_in_current_process("cleanup_finished_threads", cleanup_finished_threads);
}

pub(crate) fn spawn(thread: Thread<Ready>) {
    new_threads().push(thread)
}

extern "C" fn cleanup_finished_threads() {
    let queue = finished_threads();
    loop {
        match queue.pop() {
            Some(thread) => {
                process_tree().write().remove_thread(thread.process().pid(), thread.id());
                free_thread(thread);
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

pub(crate) unsafe fn exit_current_thread() -> ! {
    unsafe { scheduler().exit_current_thread() }
}

pub struct Scheduler {
    current_thread: Thread<Running>,
    current_thread_should_exit: AtomicBool,
    ready: VecDeque<Thread<Ready>>,
    _dummy_last_stack_ptr: usize,
}

impl Scheduler {
    /// Creates a new scheduler.
    ///
    /// # Safety
    /// Calling this more than once may result in UB due to aliasing
    /// of memory areas, threads and processes.
    unsafe fn new(kernel_thread: Thread<Running>) -> Self {
        Self {
            current_thread: kernel_thread,
            current_thread_should_exit: AtomicBool::new(false),
            ready: VecDeque::new(),
            _dummy_last_stack_ptr: 0,
        }
    }

    pub fn exit_current_thread(&self) -> ! {
        self.current_thread_should_exit.store(true, Relaxed);
        loop {
            hlt();
        }
    }

    pub fn current_thread(&self) -> &Thread<Running> {
        &self.current_thread
    }

    pub fn current_pid(&self) -> &ProcessId {
        self.current_process().pid()
    }

    pub fn current_process(&self) -> &Process {
        self.current_thread.process()
    }

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
        /*
        IMPORTANT!!!
        WE CAN NOT ACQUIRE ANY LOCKS!!!
        This will cause deadlocks and/or instability!

        Imagine the lock being held by the current thread, so we are unable
        to acquire the lock in here. This means, we won't be able to switch
        to another thread, and the current thread will never release the lock.
        */

        // move new threads to the ready queue
        while let Some(thread) = new_threads().pop() {
            self.ready.push_back(thread);
        }

        let thread = self.ready.pop_front().expect("no threads to schedule"); // this should never happen, as we should have at least the kernel thread that is in a hlt-loop (or still busy with booting)

        let cr3_value = thread.process().cr3_value();

        /*
        @dev please note that from here on, you have to enable interrupts manually if you wish to exit early
        and that this will not be done for you except if the method ends in the actual task switch
        */
        interrupts::disable(); // will be enabled again during task switch (in assembly)

        // swap out the thread from the queue and the current thread
        let mut thread = thread.into_running();
        swap(&mut self.current_thread, &mut thread);

        let should_exit = self.current_thread_should_exit.swap(false, Relaxed);
        let old_stack_ptr = if should_exit {
            let thread = thread.into_finished();
            finished_threads().push(thread);
            &mut self._dummy_last_stack_ptr as *mut usize
        } else {
            let thread = thread.into_ready();
            self.ready.push_back(thread);
            self.ready
                .back_mut()
                .unwrap()
                .last_stack_ptr_mut()
                .as_mut()
                .get_mut() as *mut usize
        };

        let new_stack_ptr = *self.current_thread.last_stack_ptr().as_ref() as *const u8;

        unsafe { switch(old_stack_ptr, new_stack_ptr, cr3_value) }
    }
}

fn free_thread(thread: Thread<Finished>) {
    serial_println!(
        "freeing thread {} ({}) in process {} ({})",
        thread.id(),
        thread.name(),
        thread.process().pid(),
        thread.process().name()
    );

    // TODO: unwind

    // TODO: deallocate stack

    let mut process_tree = process_tree().write();
    let pid = *thread.process().pid();
    process_tree.remove_thread(&pid, thread.id());
    drop(thread);

    if !process_tree.has_threads(&pid) {
        let process = match process_tree.remove_process(&pid) {
            None => {
                panic!(
                    "tried to free process {}, but process doesn't exist in the process tree",
                    pid
                );
            }
            Some(p) => p,
        };

        serial_println!("freeing process {} ({}) because it has no more threads", process.pid(), process.name());

        // TODO: deallocate address space

        drop(process);
    }
}
