use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::sync::Arc;
use core::ops::Deref;
use core::sync::atomic::AtomicU64;
use core::sync::atomic::Ordering::Relaxed;

use spin::RwLock;
use x86_64::VirtAddr;

pub use scheduler::*;
pub use tree::*;

use crate::io::path::Path;
use crate::io::vfs::{vfs, VfsError, VfsNode};
use crate::mem::virt::VirtualMemoryManager;
use crate::mem::{AddressSpace, Size};
use crate::process::attributes::{Attributes, ProcessId};
use crate::process::fd::{FileDescriptor, Fileno, FilenoAllocator};
use crate::process::task::{Ready, Running, Task};

mod attributes;
pub mod elf;
pub mod fd;
mod scheduler;
mod task;
mod tree;

pub fn init(root_process: Process) {
    let current_task = unsafe { Task::kernel_task(root_process.clone()) };
    let mut pt_guard = process_tree().write();
    pt_guard.set_root(root_process.clone());
    pt_guard.add_task(root_process.pid(), current_task.task_id());

    scheduler::init(current_task);
}

pub fn create(parent: Process, name: impl Into<String>) -> Process {
    let address_space = AddressSpace::allocate_new();
    let process = Process::new(name, address_space);

    process_tree()
        .write()
        .insert_process(parent, process.clone());

    process
}

pub fn current() -> &'static Process {
    current_task().process()
}

pub fn vmm() -> &'static VirtualMemoryManager {
    current().vmm()
}

pub fn current_task() -> &'static Task<Running> {
    unsafe { scheduler() }.current_task()
}

pub fn spawn_task_in_current_process(name: impl Into<String>, func: extern "C" fn()) {
    spawn_task(name, current(), func)
}

pub fn spawn_task(name: impl Into<String>, process: &Process, func: extern "C" fn()) {
    let task = Task::<Ready>::new(process, name, func);
    spawn(task)
}

pub fn exit() -> ! {
    unsafe { exit_current_task() }
}

/// A process is a ref counted rwlock over [`ProcessData`], which is the actual data
/// of the process.
#[derive(Clone, Debug)]
pub struct Process {
    cr3_value: usize, // TODO: remove this, read it from the address space (maybe use an atomic to circumvent the locking?)
    name: String,
    address_space: Arc<RwLock<AddressSpace>>,
    virtual_memory_manager: Arc<VirtualMemoryManager>,
    attributes: Arc<Attributes>,
}

impl Deref for Process {
    type Target = Attributes;

    fn deref(&self) -> &Self::Target {
        &self.attributes
    }
}

impl Process {
    pub fn new(name: impl Into<String>, address_space: AddressSpace) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let pid = COUNTER.fetch_add(1, Relaxed).into();

        Self {
            cr3_value: address_space.cr3_value(),
            name: name.into(),
            address_space: Arc::new(RwLock::new(address_space)),
            virtual_memory_manager: Arc::new(unsafe {
                VirtualMemoryManager::new(VirtAddr::new(0x1111_1111_0000), Size::TiB(100).bytes())
            }),
            attributes: Arc::new({
                let mut builder = Attributes::builder();
                // TODO: set attributes correctly
                builder
                    .pid(pid)
                    .euid(0.into())
                    .egid(0.into())
                    .uid(0.into())
                    .gid(0.into())
                    .suid(0.into())
                    .sgid(0.into())
                    .next_fd(FilenoAllocator::new())
                    .open_fds(Default::default());
                builder.build()
            }),
        }
    }

    pub fn pid(&self) -> &ProcessId {
        &self.pid
    }

    pub fn open_fds(&self) -> &RwLock<BTreeMap<Fileno, FileDescriptor>> {
        &self.open_fds
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn address_space(&self) -> &RwLock<AddressSpace> {
        &self.address_space
    }

    pub(in crate::process) fn cr3_value(&self) -> usize {
        self.cr3_value
    }

    pub fn vmm(&self) -> &VirtualMemoryManager {
        &self.virtual_memory_manager
    }

    pub fn open_file<P>(&self, path: P) -> Result<Fileno, VfsError>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let node = vfs().open(path)?;
        Ok(self.get_fileno_for(node))
    }

    pub fn get_fileno_for(&self, node: VfsNode) -> Fileno {
        let fd = self.next_fd.next();
        self.open_fds()
            .write()
            .insert(fd, FileDescriptor::new(node));
        fd
    }

    pub fn read(&self, fileno: Fileno, buf: &mut [u8]) -> Result<usize, VfsError> {
        let mut guard = self.open_fds().write();
        let fd = match guard.get_mut(&fileno) {
            Some(fd) => fd,
            None => return Err(VfsError::HandleClosed),
        };
        fd.read(buf)
    }

    pub fn read_at(
        &self,
        fileno: Fileno,
        buf: &mut [u8],
        offset: usize,
    ) -> Result<usize, VfsError> {
        let mut guard = self.open_fds().write();
        let fd = match guard.get_mut(&fileno) {
            Some(fd) => fd,
            None => return Err(VfsError::HandleClosed),
        };
        fd.read_at(buf, offset)
    }

    pub fn write(&self, fileno: Fileno, buf: &[u8]) -> Result<usize, VfsError> {
        let mut guard = self.open_fds().write();
        let fd = match guard.get_mut(&fileno) {
            Some(fd) => fd,
            None => return Err(VfsError::HandleClosed),
        };
        fd.write(buf)
    }

    pub fn close_fd(&self, fd: Fileno) -> Result<(), VfsError> {
        let descriptor = match self.open_fds().write().remove(&fd) {
            Some(fd) => fd,
            None => return Err(VfsError::HandleClosed),
        };

        // close the actual file
        let node = descriptor.into_node();
        drop(node);
        Ok(())
    }
}
