use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::ptr;

use conquer_once::spin::OnceCell;
use kernel_vfs::path::{AbsoluteOwnedPath, AbsolutePath, ROOT};
use spin::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use virtual_memory_manager::VirtualMemoryManager;
use x86_64::VirtAddr;

use crate::mcore::mtask::process;
use crate::mcore::mtask::process::fd::{FdNum, FileDescriptor};
use crate::mcore::mtask::process::{CreateProcessError, Process, ProcessId, ROOT_PROCESS};
use crate::mcore::mtask::scheduler::global::GlobalTaskQueue;
use crate::mcore::mtask::task::{Stack, StackUserAccessible, Task};
use crate::mem::address_space::AddressSpace;
use crate::mem::virt::{VirtualMemoryAllocator, VirtualMemoryHigherHalf};

static PROCESS_TREE: OnceCell<RwLock<ProcessTree>> = OnceCell::uninit();

pub fn process_tree() -> &'static RwLock<ProcessTree> {
    PROCESS_TREE.get_or_init(|| {
        RwLock::new(ProcessTree {
            children: BTreeMap::default(),
            processes: BTreeMap::default(),
        })
    })
}

pub struct ProcessTree {
    pub children: BTreeMap<ProcessId, Vec<Arc<Process>>>,
    pub processes: BTreeMap<ProcessId, Arc<Process>>,
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

impl Process {
    pub fn root() -> &'static Arc<Process> {
        ROOT_PROCESS.get_or_init(|| {
            let pid = ProcessId::new();
            let root = Arc::new(Self {
                pid,
                name: "root".to_string(),
                ppid: RwLock::new(pid),
                executable_path: None,
                current_working_directory: RwLock::new(ROOT.to_owned()),
                address_space: None,
                lower_half_memory: Arc::new(RwLock::new(VirtualMemoryManager::new(
                    VirtAddr::new(0x00),
                    0x0000_7FFF_FFFF_FFFF,
                ))),
                file_descriptors: RwLock::new(BTreeMap::new()),
            });
            process_tree().write().processes.insert(pid, root.clone());
            root
        })
    }

    fn create_new(
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
            current_working_directory: RwLock::new(parent.current_working_directory.read().clone()),
            address_space: Some(address_space),
            lower_half_memory: Arc::new(RwLock::new(VirtualMemoryManager::new(
                VirtAddr::new(0xF000),
                0x0000_7FFF_FFFF_0FFF,
            ))),
            file_descriptors: RwLock::new(BTreeMap::new()),
        };

        let res = Arc::new(process);
        process_tree().write().processes.insert(pid, res.clone());
        res
    }

    // TODO: add documentation
    #[allow(clippy::missing_errors_doc)]
    pub fn create_from_executable(
        parent: &Arc<Process>,
        path: impl AsRef<AbsolutePath>,
    ) -> Result<Arc<Self>, CreateProcessError> {
        let path = path.as_ref();
        let process = Self::create_new(parent, path.to_string(), Some(path));

        let kstack = Stack::allocate(
            16,
            &VirtualMemoryHigherHalf,
            StackUserAccessible::No,
            AddressSpace::kernel(),
            process::trampoline,
            ptr::null_mut(),
            Task::exit,
        )?;
        let main_task = Task::create_with_stack(&process, kstack);
        GlobalTaskQueue::enqueue(Box::pin(main_task));

        Ok(process)
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

    pub fn file_descriptors(&self) -> &RwLock<BTreeMap<FdNum, FileDescriptor>> {
        &self.file_descriptors
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

    pub fn current_working_directory(&self) -> &RwLock<AbsoluteOwnedPath> {
        &self.current_working_directory
    }
}
