use crate::mcore::mtask::process::tree::{process_tree, ProcessTree};
use crate::mem::address_space::AddressSpace;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use conquer_once::spin::OnceCell;
use core::fmt::{Debug, Formatter};
use spin::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use virtual_memory_manager::VirtualMemoryManager;
use x86_64::VirtAddr;

pub use id::*;

mod id;
mod tree;

static ROOT_PROCESS: OnceCell<Arc<Process>> = OnceCell::uninit();

pub struct Process {
    pid: ProcessId,
    name: String,

    ppid: RwLock<ProcessId>,

    address_space: Option<AddressSpace>,
    _lower_half_memory: RwLock<VirtualMemoryManager>,
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

impl Process {
    pub fn root() -> &'static Arc<Process> {
        ROOT_PROCESS.get_or_init(|| {
            let pid = ProcessId::new();
            let root = Arc::new(Process {
                pid,
                name: "root".to_string(),
                ppid: RwLock::new(pid),
                address_space: None,
                _lower_half_memory: RwLock::new(VirtualMemoryManager::new(
                    VirtAddr::new(0x00),
                    0x0000_7FFF_FFFF_FFFF,
                )),
            });
            process_tree().write().processes.insert(pid, root.clone());
            root
        })
    }

    pub fn create_new(parent: &Arc<Process>, name: String) -> Arc<Self> {
        let pid = ProcessId::new();
        let parent_pid = parent.pid;
        let address_space = AddressSpace::new();

        let process = Self {
            pid,
            name,
            ppid: RwLock::new(parent_pid),
            address_space: Some(address_space),
            _lower_half_memory: RwLock::new(VirtualMemoryManager::new(
                VirtAddr::new(0xF000),
                0x0000_7FFF_FFFF_0FFF,
            )),
        };

        let res = Arc::new(process);
        process_tree().write().processes.insert(pid, res.clone());
        res
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
