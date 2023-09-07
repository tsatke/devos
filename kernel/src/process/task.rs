use crate::process::Process;
use core::marker::PhantomData;
use core::sync::atomic::AtomicU64;
use core::sync::atomic::Ordering::Relaxed;
use derive_more::Display;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Display)]
pub struct TaskId(u64);

impl !Default for TaskId {}

impl TaskId {
    pub fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        TaskId(COUNTER.fetch_add(1, Relaxed))
    }
}

pub trait State {}

macro_rules! state {
    ($($name:ident),*) => {
        $(
            #[derive(Copy, Clone, Debug, derive_more::Display, Eq, PartialEq)]
            pub struct $name;
            impl State for $name {}
        )*
    };
}

state!(Ready, Running, Finished);

#[derive(Debug)]
pub struct Task<S>
where
    S: State + 'static,
{
    id: TaskId,
    process: Process,
    last_stack_ptr: usize,
    _state: PhantomData<S>,
}

impl<S> PartialEq<Self> for Task<S>
where
    S: 'static + State,
{
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.last_stack_ptr == other.last_stack_ptr
    }
}

impl<S> Eq for Task<S> where S: State + 'static {}

impl<S> Task<S>
where
    S: State + 'static,
{
    pub fn task_id(&self) -> &TaskId {
        &self.id
    }

    pub fn last_stack_ptr(&self) -> &usize {
        &self.last_stack_ptr
    }

    pub fn process(&self) -> &Process {
        &self.process
    }

    pub fn last_stack_ptr_mut(&mut self) -> &mut usize {
        &mut self.last_stack_ptr
    }
}

impl From<Task<Ready>> for Task<Running> {
    fn from(value: Task<Ready>) -> Self {
        Task {
            id: value.id,
            process: value.process,
            last_stack_ptr: value.last_stack_ptr,
            _state: Default::default(),
        }
    }
}

impl From<Task<Running>> for Task<Finished> {
    fn from(value: Task<Running>) -> Self {
        Task {
            id: value.id,
            process: value.process,
            last_stack_ptr: value.last_stack_ptr,
            _state: Default::default(),
        }
    }
}

impl From<Task<Running>> for Task<Ready> {
    fn from(value: Task<Running>) -> Self {
        Task {
            id: value.id,
            process: value.process,
            last_stack_ptr: value.last_stack_ptr,
            _state: Default::default(),
        }
    }
}

impl Task<Ready> {
    pub fn into_running(self) -> Task<Running> {
        self.into()
    }
}

impl Task<Running> {
    pub unsafe fn kernel_task(kernel_process: Process) -> Self {
        Self {
            id: TaskId::new(),
            process: kernel_process,
            last_stack_ptr: 0, // will be set correctly during the next `reschedule`
            _state: Default::default(),
        }
    }

    pub fn into_finished(self) -> Task<Finished> {
        self.into()
    }

    pub fn into_ready(self) -> Task<Ready> {
        self.into()
    }
}
