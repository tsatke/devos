use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::fmt::Debug;
use core::marker::PhantomData;
use core::mem::size_of;
use core::sync::atomic::AtomicU64;
use core::sync::atomic::Ordering::Relaxed;

use derive_more::Display;
use x86_64::registers::rflags::RFlags;

use crate::mem::Size;
use crate::process;
use crate::process::Process;

const STACK_SIZE: usize = Size::KiB(32).bytes();

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Display)]
pub struct ThreadId(u64);

impl<T> PartialEq<T> for ThreadId
where
    T: Into<u64> + Copy,
{
    fn eq(&self, other: &T) -> bool {
        self.0 == (*other).into()
    }
}

impl !Default for ThreadId {}

impl ThreadId {
    pub fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        ThreadId(COUNTER.fetch_add(1, Relaxed))
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
        impl From<Thread<$from>> for Thread<$to> {
            fn from(value: Thread<$from>) -> Self {
                Thread {
                    id: value.id,
                    name: value.name,
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
pub struct Thread<S>
where
    S: State + 'static,
{
    id: ThreadId,
    name: String,
    process: Process,
    last_stack_ptr: usize,
    stack: Option<Vec<u8>>,
    _state: PhantomData<S>,
}

impl<S> PartialEq<Self> for Thread<S>
where
    S: 'static + State,
{
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.last_stack_ptr == other.last_stack_ptr
    }
}

impl<S> Eq for Thread<S> where S: State + 'static {}

impl<S> Thread<S>
where
    S: State + 'static,
{
    pub fn task_id(&self) -> &ThreadId {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
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

impl Thread<Ready> {
    pub fn new(
        process: &Process,
        name: impl Into<String>,
        entry_point: extern "C" fn(),
    ) -> Thread<Ready> {
        let mut task = Self {
            id: ThreadId::new(),
            name: name.into(),
            process: process.clone(),
            last_stack_ptr: 0, // will be set correctly in [`setup_stack`]
            stack: Some(vec![0; STACK_SIZE]),
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

    pub fn into_running(self) -> Thread<Running> {
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
    unsafe { process::exit_current_task() }
}

impl Thread<Running> {
    pub unsafe fn kernel_task(kernel_process: Process) -> Self {
        Self {
            id: ThreadId::new(),
            name: "kernel".to_string(),
            process: kernel_process,
            last_stack_ptr: 0, // will be set correctly during the next `reschedule`
            stack: None, // FIXME: use the correct stack on the heap (obtained through the bootloader)
            _state: Default::default(),
        }
    }

    pub fn into_finished(self) -> Thread<Finished> {
        self.into()
    }

    pub fn into_ready(self) -> Thread<Ready> {
        self.into()
    }
}
