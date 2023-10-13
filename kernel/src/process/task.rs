use crate::mem::Size;
use crate::process::Process;
use alloc::boxed::Box;
use core::fmt::Debug;
use core::marker::PhantomData;
use core::mem::size_of;
use core::sync::atomic::AtomicU64;
use core::sync::atomic::Ordering::Relaxed;
use derive_more::Display;
use x86_64::registers::rflags::RFlags;

const STACK_SIZE: usize = Size::KiB(32).bytes();

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

macro_rules! state_transition {
    ($from:ident, $to:ident) => {
        impl From<Task<$from>> for Task<$to> {
            fn from(value: Task<$from>) -> Self {
                Task {
                    id: value.id,
                    process: value.process,
                    last_stack_ptr: value.last_stack_ptr,
                    stack: value.stack,
                    _state: Default::default(),
                }
            }
        }
    };
}

state!(Ready, Running, Finished);
state_transition!(Ready, Running);
state_transition!(Running, Finished);
state_transition!(Running, Ready);

#[derive(Debug)]
pub struct Task<S>
where
    S: State + 'static,
{
    id: TaskId,
    process: Process,
    last_stack_ptr: usize,
    stack: Option<Box<[u8; STACK_SIZE]>>,
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

struct StackWriter<'a> {
    i: usize,
    data: &'a mut [u8],
}

impl<'a> StackWriter<'a> {
    fn new(data: &'a mut [u8]) -> Self {
        Self {
            i: data.len(),
            data,
        }
    }

    fn write_u64(&mut self, value: u64) {
        let data = value.to_ne_bytes();
        let len = data.len();
        self.data[self.i..self.i + len].copy_from_slice(&data[..]);
    }

    fn write_task_state(&mut self, val: TaskState) {
        let data = unsafe { core::mem::transmute::<TaskState, [u8; size_of::<TaskState>()]>(val) };
        let len = data.len();
        self.data[self.i..self.i + len].copy_from_slice(&data[..]);
    }

    fn back_qword(&mut self) {
        self.back_n(size_of::<u64>());
    }

    fn back_n(&mut self, n: usize) {
        self.i -= n;
    }
}

impl Task<Ready> {
    pub fn new(process: Process, entry_point: extern "C" fn()) -> Task<Ready> {
        let mut task = Self {
            id: TaskId::new(),
            process,
            last_stack_ptr: 0, // will be set correctly in [`setup_stack`]
            stack: Some(Box::new([0; STACK_SIZE])),
            _state: Default::default(),
        };
        task.setup_stack(entry_point);
        task
    }

    fn setup_stack(&mut self, entry_point: extern "C" fn()) {
        let entry_point = entry_point as *const () as *const usize;
        let stack = self
            .stack
            .as_mut()
            .expect("can't initialize a task without stack");
        stack.fill(0xCD); // fill the stack with 0xCD

        let mut writer = StackWriter::new(stack.as_mut_slice());
        writer.back_qword();
        writer.write_u64(0xDEADCAFEBEEFBABEu64); // marker at stack bottom

        // TODO: add arguments for the function

        writer.back_qword();
        writer.write_u64(leave_task as *const () as u64); // put return address on the stack

        writer.back_n(size_of::<TaskState>());
        // remember where we are right now
        let rsp = writer.data.as_ptr() as u64 + writer.i as u64; // the stack starts before the task state struct in memory (stack grows backwards)
        writer.write_task_state(TaskState {
            rsp, // stack pointer points to the state (stack grows backwards)
            // rbp: rsp + size_of::<u64>() as u64, // base pointer points to the stack pointer
            rbp: rsp,                // base pointer points to the stack pointer
            rip: entry_point as u64, // push the entry point as the instruction pointer
            rflags: (RFlags::IOPL_LOW | RFlags::INTERRUPT_FLAG).bits(),
            ..Default::default()
        });

        self.last_stack_ptr = rsp as usize;
    }

    pub fn into_running(self) -> Task<Running> {
        self.into()
    }
}

#[repr(C, packed)]
#[derive(Default, Debug)]
pub struct TaskState {
    r15: u64,
    r14: u64,
    r13: u64,
    r12: u64,
    r11: u64,
    r10: u64,
    r9: u64,
    r8: u64,
    rdi: u64,
    rsi: u64,
    rsp: u64,
    rbp: u64,
    rdx: u64,
    rcx: u64,
    rbx: u64,
    rax: u64,
    rflags: u64,
    rip: u64,
}

extern "C" fn leave_task() -> ! {
    todo!("leave task")
}

impl Task<Running> {
    pub unsafe fn kernel_task(kernel_process: Process) -> Self {
        Self {
            id: TaskId::new(),
            process: kernel_process,
            last_stack_ptr: 0, // will be set correctly during the next `reschedule`
            stack: None, // FIXME: use the correct stack on the heap (obtained through the bootloader)
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
