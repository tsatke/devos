use alloc::boxed::Box;
use core::array::IntoIter;
use core::ffi::c_void;
use core::iter::Cycle;
use core::ptr;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering::Relaxed;

use x86_64::instructions::hlt;

pub use queues::Priority;

use crate::process::attributes::ProcessId;
use crate::process::scheduler::lfill::LockFreeIntrusiveLinkedList;
use crate::process::scheduler::queues::{AtomicPriority, Queues};
use crate::process::scheduler::thread::{State, Thread};
use crate::process::Priority::{High, Low, Normal, Realtime};
use crate::process::{process_tree, spawn_thread_in_current_process, Process};
use crate::serial_println;

pub mod lfill;
mod queues;
mod reschedule;
pub mod thread;

static mut SCHEDULER: Option<Scheduler> = None;

pub static IN_RESCHEDULE: AtomicBool = AtomicBool::new(false);

// this needs to be a lock-free, allocation-free list, because the scheduler appends to it
static FINISHED_THREADS: LockFreeIntrusiveLinkedList<Thread> = LockFreeIntrusiveLinkedList::new();
// this needs to be a lock-free, allocation-free list, because the scheduler reads from it
static NEW_THREADS: LockFreeIntrusiveLinkedList<Thread> = LockFreeIntrusiveLinkedList::new();

pub fn init(kernel_thread: Thread) {
    unsafe { SCHEDULER = Some(Scheduler::new(kernel_thread)) };

    // now that the finish queue is initialized, we can spawn the cleanup thread
    spawn_thread_in_current_process(
        "cleanup_finished_threads",
        Low,
        cleanup_finished_threads,
        ptr::null_mut(),
    );
}

pub(crate) fn spawn(thread: Thread) {
    assert_eq!(thread.state(), State::Ready);
    NEW_THREADS.push_back(Box::into_raw(Box::new(thread)))
}

extern "C" fn cleanup_finished_threads(_: *mut c_void) {
    loop {
        match FINISHED_THREADS.pop_front() {
            Some(thread) => {
                let thread = *unsafe { Box::from_raw(thread) };
                process_tree()
                    .write()
                    .remove_thread(thread.process().pid(), thread.id());
                free_thread(thread);
            }
            None => {
                hlt();
                continue;
            }
        }
    }
}

#[allow(static_mut_refs)] // we know what we're doing
pub(in crate::process) unsafe fn scheduler() -> &'static Scheduler {
    SCHEDULER.as_ref().unwrap()
}

#[allow(static_mut_refs)] // we know what we're doing
pub(in crate::process) unsafe fn scheduler_mut() -> &'static mut Scheduler {
    SCHEDULER.as_mut().unwrap()
}

/// # Safety
/// This can only be called after [`init`] has been called.
/// This may or may not return, make sure to use it in a way that can handle both cases.
pub(crate) unsafe fn reschedule() {
    unsafe { scheduler_mut().reschedule() }
}

pub(crate) unsafe fn change_current_thread_prio(prio: Priority) {
    unsafe { scheduler().change_current_thread_prio(prio) }
}

pub(crate) unsafe fn exit_current_thread() -> ! {
    unsafe { scheduler().exit_current_thread() }
}

const STRATEGY_LENGTH: usize = 10;

pub struct Scheduler {
    current_thread: Box<Thread>,
    current_thread_should_exit: AtomicBool,
    current_thread_prio: AtomicPriority,
    strategy: Cycle<IntoIter<Priority, STRATEGY_LENGTH>>,
    ready: Queues<LockFreeIntrusiveLinkedList<Thread>>,
    _dummy_last_stack_ptr: usize,
}

impl Scheduler {
    /// Creates a new scheduler.
    ///
    /// # Safety
    /// Calling this more than once may result in UB due to aliasing
    /// of memory areas, threads and processes.
    unsafe fn new(kernel_thread: Thread) -> Self {
        let priority = kernel_thread.priority();
        Self {
            current_thread: Box::new(kernel_thread),
            current_thread_should_exit: AtomicBool::new(false),
            current_thread_prio: AtomicPriority::new(priority),
            strategy: [
                Realtime, High, Normal, Realtime, High, Low, Realtime, High, Realtime, Normal,
            ]
            .into_iter()
            .cycle(),
            ready: Queues::new(
                LockFreeIntrusiveLinkedList::new(),
                LockFreeIntrusiveLinkedList::new(),
                LockFreeIntrusiveLinkedList::new(),
                LockFreeIntrusiveLinkedList::new(),
            ),
            _dummy_last_stack_ptr: 0,
        }
    }

    pub fn exit_current_thread(&self) -> ! {
        self.current_thread_should_exit.store(true, Relaxed);
        loop {
            hlt();
        }
    }

    pub fn change_current_thread_prio(&self, prio: Priority) {
        self.current_thread_prio.store(prio, Relaxed);
    }

    pub fn current_thread(&self) -> &Thread {
        &self.current_thread
    }

    pub fn current_pid(&self) -> &ProcessId {
        self.current_process().pid()
    }

    pub fn current_process(&self) -> &Process {
        self.current_thread.process()
    }
}

fn free_thread(thread: Thread) {
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

        serial_println!(
            "freeing process {} ({}) because it has no more threads",
            process.pid(),
            process.name()
        );

        // TODO: deallocate address space

        drop(process);
    }
}
