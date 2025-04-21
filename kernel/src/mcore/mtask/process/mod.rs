use crate::mcore::mtask::process::tree::{process_tree, ProcessTree};
use crate::mem::address_space::AddressSpace;
use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use conquer_once::spin::OnceCell;
use core::ffi::c_void;
use core::fmt::{Debug, Formatter};
use core::mem::transmute;
use core::ptr;
use elfloader::ElfBinary;
use log::{debug, info};
use spin::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use thiserror::Error;
use virtual_memory_manager::VirtualMemoryManager;
use x86_64::registers::rflags::RFlags;
use x86_64::registers::segmentation::SegmentSelector;
use x86_64::VirtAddr;

use crate::mcore::context::ExecutionContext;
use crate::mcore::mtask::process::elf::ElfLoader;
use crate::mcore::mtask::process::iretq::IretqFrame;
use crate::mcore::mtask::scheduler::global::GlobalTaskQueue;
use crate::mcore::mtask::task::{Stack, StackAllocationError, Task};
use crate::mem::virt::{VirtualMemoryAllocator, VirtualMemoryHigherHalf};
use crate::vfs::vfs;
pub use id::*;
use vfs::path::{AbsoluteOwnedPath, AbsolutePath};

mod elf;
mod id;
mod iretq;
mod tree;

static ROOT_PROCESS: OnceCell<Arc<Process>> = OnceCell::uninit();

pub struct Process {
    pid: ProcessId,
    name: String,

    ppid: RwLock<ProcessId>,

    executable_path: Option<AbsoluteOwnedPath>,

    address_space: Option<AddressSpace>,
    lower_half_memory: Arc<RwLock<VirtualMemoryManager>>,
}

impl Debug for Process {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Process")
            .field("pid", &self.pid)
            .field("ppid", &*self.ppid.read())
            .field("name", &self.name)
            .field("address_space", self.address_space())
            .finish_non_exhaustive()
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        let my_ppid = *self.ppid.read();
        let mut guard = process_tree().write();
        guard
            .processes
            .remove(&self.pid)
            .expect("process should be in process tree");
        if let Some(children) = guard.children.remove(&self.pid) {
            for child in children {
                *child.ppid.write() = my_ppid;
            }
        }

        // TODO: deallocate all physical frames that are not part of a shared mapping
    }
}

#[derive(Debug, Error)]
pub enum CreateProcessError {
    #[error("failed to allocate stack")]
    StackAllocationError(#[from] StackAllocationError),
}

impl Process {
    pub fn root() -> &'static Arc<Process> {
        ROOT_PROCESS.get_or_init(|| {
            let pid = ProcessId::new();
            let root = Arc::new(Process {
                pid,
                name: "root".to_string(),
                ppid: RwLock::new(pid),
                executable_path: None,
                address_space: None,
                lower_half_memory: Arc::new(RwLock::new(VirtualMemoryManager::new(
                    VirtAddr::new(0x00),
                    0x0000_7FFF_FFFF_FFFF,
                ))),
            });
            process_tree().write().processes.insert(pid, root.clone());
            root
        })
    }

    pub fn create_new(
        parent: &Arc<Process>,
        name: String,
        executable_path: Option<impl AsRef<AbsolutePath>>,
    ) -> Arc<Self> {
        let pid = ProcessId::new();
        let parent_pid = parent.pid;
        let address_space = AddressSpace::new();

        let process = Self {
            pid,
            name,
            ppid: RwLock::new(parent_pid),
            executable_path: executable_path.map(|x| x.as_ref().to_owned()),
            address_space: Some(address_space),
            lower_half_memory: Arc::new(RwLock::new(VirtualMemoryManager::new(
                VirtAddr::new(0xF000),
                0x0000_7FFF_FFFF_0FFF,
            ))),
        };

        let res = Arc::new(process);
        process_tree().write().processes.insert(pid, res.clone());
        res
    }

    pub fn pid(&self) -> ProcessId {
        self.pid
    }

    pub fn ppid(&self) -> ProcessId {
        *self.ppid.read()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    #[allow(clippy::missing_panics_doc)] // this panic must not happen, so the caller shouldn't have to care about it
    pub fn parent(&self) -> Arc<Process> {
        process_tree()
            .read()
            .processes
            .get(&*self.ppid.read())
            .expect("parent process not found")
            .clone()
    }

    pub fn children(&self) -> Children<'_> {
        let guard = process_tree().read();
        Children {
            guard,
            pid: self.pid,
        }
    }

    pub fn children_mut(&self) -> ChildrenMut<'_> {
        let guard = process_tree().write();
        ChildrenMut {
            guard,
            pid: self.pid,
        }
    }

    pub fn address_space(&self) -> &AddressSpace {
        self.address_space
            .as_ref()
            .unwrap_or(AddressSpace::kernel())
    }

    pub fn vmm(self: &Arc<Self>) -> impl VirtualMemoryAllocator {
        self.lower_half_memory.clone()
    }
}

impl Process {
    pub fn create_from_executable(
        parent: &Arc<Process>,
        path: impl AsRef<AbsolutePath>,
    ) -> Result<Arc<Self>, CreateProcessError> {
        let path = path.as_ref();
        let process = Self::create_new(parent, path.to_string(), Some(path));

        let kstack = Stack::allocate(
            16,
            VirtualMemoryHigherHalf,
            AddressSpace::kernel(),
            trampoline,
            ptr::null_mut(),
            Task::exit,
        )?;
        let main_task = Task::create_with_stack(&process, kstack);
        GlobalTaskQueue::enqueue(Box::pin(main_task));

        Ok(process)
    }
}

extern "C" fn trampoline(_arg: *mut c_void) {
    info!("trampoline called");

    let ctx = ExecutionContext::load();
    let current_task = ctx.scheduler().current_task();
    let current_process = current_task.process().clone();

    let executable_path = current_process
        .executable_path
        .as_ref()
        .expect("should have an executable path");
    let node = vfs()
        .write()
        .open(executable_path)
        .expect("should be able to open executable");

    let mut data = Vec::new();
    let mut buf = [0; 4096];
    let mut offset = 0;
    loop {
        let read = node.read(&mut buf, offset).expect("should be able to read");
        if read == 0 {
            break;
        }
        offset += read;
        data.extend_from_slice(&buf[..read]);
    }

    let elf_binary = ElfBinary::new(&data).expect("should be able to parse elf binary");
    let mut elf_loader = ElfLoader::new(current_process.clone());
    elf_binary
        .load(&mut elf_loader)
        .expect("should be able to load elf binary");
    let image = elf_loader.into_inner();
    let code_ptr = unsafe { image.as_ptr().add(elf_binary.entry_point() as usize) };
    let entry_fn = unsafe { transmute(code_ptr) };

    // TODO: set up stack pointer, return address etc for an iretq

    let ustack = Stack::allocate(
        256,
        current_process.vmm(),
        current_process.address_space(),
        entry_fn,
        ptr::null_mut(),
        Task::exit,
    )
    .expect("should be able to allocate userspace stack");
    let ustack_rsp = ustack.initial_rsp();
    {
        let mut ustack_guard = current_task.ustack().write();
        assert!(ustack_guard.is_none(), "ustack should not exist yet");
        *ustack_guard = Some(ustack);
    }

    let sel = ctx.selectors();
    let iretq_frame = IretqFrame {
        stack_segment: sel.user_data,
        stack_pointer: ustack_rsp,
        rflags: RFlags::INTERRUPT_FLAG,
        code_segment: sel.user_code,
        instruction_pointer: VirtAddr::new(code_ptr as u64),
    };
    unsafe {
        iretq_frame.iretq();
    }
}

pub struct Children<'a> {
    guard: RwLockReadGuard<'a, ProcessTree>,
    pid: ProcessId,
}

impl Children<'_> {
    #[must_use]
    pub fn get(&self) -> Option<impl Iterator<Item = &Arc<Process>>> {
        self.guard.children.get(&self.pid).map(|x| x.iter())
    }
}

pub struct ChildrenMut<'a> {
    guard: RwLockWriteGuard<'a, ProcessTree>,
    pid: ProcessId,
}

impl ChildrenMut<'_> {
    pub fn get_mut(&mut self) -> Option<impl Iterator<Item = &mut Arc<Process>>> {
        self.guard.children.get_mut(&self.pid).map(|x| x.iter_mut())
    }

    pub fn insert(&mut self, process: Arc<Process>) {
        self.guard
            .children
            .entry(self.pid)
            .or_default()
            .push(process);
    }
}
