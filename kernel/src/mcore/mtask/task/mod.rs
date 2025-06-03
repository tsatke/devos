use crate::U64Ext;
use crate::mcore::context::ExecutionContext;
use crate::mcore::mtask::process::Process;
use crate::mem::address_space::AddressSpace;
use crate::mem::memapi::{LowerHalfAllocation, Writable};
use crate::mem::virt::VirtualMemoryHigherHalf;
use alloc::boxed::Box;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use cordyceps::Linked;
use cordyceps::mpsc_queue::Links;
use core::ffi::c_void;
use core::pin::Pin;
use core::ptr::NonNull;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering::Relaxed;
pub use id::*;
use log::trace;
pub use queue::*;
use spin::RwLock;
pub use stack::*;
pub use state::*;
use x86_64::instructions::hlt;

mod id;
mod queue;
mod stack;
mod state;

#[derive(Debug)]
pub struct Task {
    /// The unique identifier of the task.
    tid: TaskId,
    /// The name of the task, not necessarily unique.
    name: String,
    /// The parent process that this task belongs to.
    /// If upon rescheduling, the parent process is not alive, the task will be terminated.
    process: Arc<Process>,
    /// Whether this task should be terminated upon the next reschedule.
    /// This can be set at any point.
    should_terminate: AtomicBool,
    /// The stack pointer of the task at the time of the last context switch.
    /// If this task is currently running, then this value is not the current stack pointer.
    /// This must be set during the context switch.
    last_stack_ptr: Pin<Box<usize>>,
    state: State,
    /// The kernel stack of the task. Every task starts with a stack in the higher half.
    /// Userspace tasks will then allocate a stack in the lower half, which will be stored in
    /// `ustack`.
    kstack: Option<Stack>,
    /// The user stack of the task. This is only set if the task is a userspace task.
    ustack: RwLock<Option<Stack>>,

    tls: RwLock<Option<LowerHalfAllocation<Writable>>>,

    links: Links<Self>,
}

impl Unpin for Task {}

unsafe impl Linked<Links<Self>> for Task {
    type Handle = Pin<Box<Self>>;

    fn into_ptr(r: Self::Handle) -> NonNull<Self> {
        NonNull::from(Box::leak(Pin::into_inner(r)))
    }

    unsafe fn from_ptr(ptr: NonNull<Self>) -> Self::Handle {
        unsafe { Pin::new(Box::from_raw(ptr.as_ptr())) }
    }

    unsafe fn links(ptr: NonNull<Self>) -> NonNull<Links<Self>> {
        let links = unsafe { &raw mut (*ptr.as_ptr()).links };
        unsafe { NonNull::new_unchecked(links) }
    }
}

impl Task {
    /// Creates a new stack in the specified process. Stack will be allocated immediately in the
    /// current address space.
    ///
    /// # Errors
    /// Returns an error if the stack could not be allocated.
    pub fn create_new(
        process: &Arc<Process>,
        entry_point: extern "C" fn(*mut c_void),
        arg: *mut c_void,
    ) -> Result<Self, StackAllocationError> {
        let stack = Stack::allocate(
            16,
            &VirtualMemoryHigherHalf,
            StackUserAccessible::No,
            AddressSpace::kernel(),
            entry_point,
            arg,
            Self::exit,
        )?;
        Ok(Self::create_with_stack(process, stack))
    }

    pub fn create_with_stack(process: &Arc<Process>, stack: Stack) -> Self {
        let tid = TaskId::new();
        let name = format!("task-{tid}");
        let process = process.clone();
        let should_terminate = AtomicBool::new(false);
        let state = State::Ready;
        let last_stack_ptr = Box::pin(stack.initial_rsp().as_u64().into_usize());
        let links = Links::default();
        Self {
            tid,
            name,
            process,
            should_terminate,
            last_stack_ptr,
            state,
            kstack: Some(stack),
            ustack: RwLock::new(None),
            tls: RwLock::new(None),
            links,
        }
    }

    pub(in crate::mcore::mtask) fn create_stub() -> Self {
        let tid = TaskId::new();
        let name = "stub".to_string();
        let process = Process::root().clone();
        let should_terminate = AtomicBool::new(false);
        let last_stack_ptr = Box::pin(0);
        let state = State::Finished;
        let stack = None;
        let links = Links::new_stub();
        Self {
            tid,
            name,
            process,
            should_terminate,
            last_stack_ptr,
            state,
            kstack: stack,
            ustack: RwLock::new(None),
            tls: RwLock::new(None),
            links,
        }
    }

    pub(crate) extern "C" fn exit() {
        let task = ExecutionContext::load().current_task();
        trace!("exiting task {}", task.name());

        task.set_should_terminate(true);
        loop {
            hlt();
        }
    }

    /// Creates a Task struct for the current state of the CPU.
    /// The task is inactive, and its values must be set by the scheduler
    /// first.
    ///
    /// The resulting task will belong to the root process.
    ///
    /// # Safety
    /// The caller must ensure that this is only called once per core.
    #[must_use]
    pub unsafe fn create_current() -> Self {
        let tid = TaskId::new();
        let name = format!("task-{tid}");
        let process = Process::root().clone();
        let should_terminate = AtomicBool::new(false);
        let last_stack_ptr = Box::pin(0);
        let state = State::Running;
        let stack = None;
        Self {
            tid,
            name,
            process,
            should_terminate,
            last_stack_ptr,
            state,
            kstack: stack,
            ustack: RwLock::new(None),
            tls: RwLock::new(None),
            links: Links::default(),
        }
    }

    pub fn id(&self) -> TaskId {
        self.tid
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn process(&self) -> &Arc<Process> {
        &self.process
    }

    pub fn should_terminate(&self) -> bool {
        self.should_terminate.load(Relaxed)
    }

    pub fn set_should_terminate(&self, should_terminate: bool) {
        self.should_terminate.store(should_terminate, Relaxed);
    }

    pub fn state(&self) -> State {
        self.state
    }

    pub fn kstack(&self) -> &Option<Stack> {
        &self.kstack
    }

    pub fn ustack(&self) -> &RwLock<Option<Stack>> {
        &self.ustack
    }

    pub fn tls(&self) -> &RwLock<Option<LowerHalfAllocation<Writable>>> {
        &self.tls
    }

    pub fn last_stack_ptr(&mut self) -> &mut usize {
        self.last_stack_ptr.as_mut().get_mut()
    }
}
