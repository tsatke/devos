use alloc::borrow::ToOwned;
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use core::intrinsics::transmute;
use core::slice::from_raw_parts;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering::Relaxed;

use elfloader::ElfBinary;
use spin::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use x86_64::structures::paging::PageTableFlags;
use x86_64::VirtAddr;

use kernel_api::syscall::Stat;
pub use scheduler::*;
pub use tree::*;

use crate::io::path::{OwnedPath, Path};
use crate::io::vfs::{vfs, VfsError, VfsNode};
use crate::mem::{AddressSpace, Size};
use crate::mem::virt::{MapAt, VirtualMemoryManager};
use crate::process::attributes::{Attributes, ProcessId, RealGroupId, RealUserId};
use crate::process::elf::ElfLoader;
use crate::process::fd::{FileDescriptor, Fileno, FilenoAllocator};
use crate::process::thread::{State, Thread};

pub mod attributes;
pub mod elf;
pub mod fd;
mod scheduler;
mod tree;

pub fn init(address_space: AddressSpace) {
    let root_process = Process::create_kernel(address_space);
    let current_thread = unsafe { Thread::kernel_thread(root_process.clone()) };
    process_tree().write().add_thread(root_process.pid(), current_thread.id());

    scheduler::init(current_thread);
}

pub fn current() -> &'static Process {
    current_thread().process()
}

pub fn vmm() -> &'static VirtualMemoryManager {
    current().vmm()
}

pub fn current_thread() -> &'static Thread {
    unsafe { scheduler() }.current_thread()
}

pub fn spawn_thread_in_current_process(name: impl Into<String>, priority: Priority, func: extern "C" fn()) {
    spawn_thread(name, current(), priority, func)
}

pub fn spawn_thread(name: impl Into<String>, process: &Process, priority: Priority, func: extern "C" fn()) {
    let thread = Thread::new_ready(process, name, priority, func);
    debug_assert_eq!(thread.state(), State::Ready);
    spawn(thread)
}

pub fn change_thread_priority(priority: Priority) {
    unsafe { change_current_thread_prio(priority) }
}

pub fn exit_thread() -> ! {
    unsafe { exit_current_thread() }
}

#[derive(Clone, Debug)]
pub struct Process {
    // TODO: remove this, read it from the address space (maybe use an atomic to circumvent the locking?)
    cr3_value: usize,
    address_space: Arc<RwLock<AddressSpace>>,
    virtual_memory_manager: Arc<VirtualMemoryManager>,

    name: String,
    pid: ProcessId,
    next_fd: Arc<FilenoAllocator>,
    open_fds: Arc<RwLock<BTreeMap<Fileno, FileDescriptor>>>,
    attributes: Arc<RwLock<Attributes>>,

    executable_file: Option<OwnedPath>,
}

extern "C" fn trampoline() {
    let proc = current();
    if proc.executable_file.is_none() {
        panic!("trampoline called for a process without an executable file");
    }
    let executable_file = proc.executable_file.as_ref().unwrap().as_path();

    let elf_data = {
        let file = vfs().open(executable_file).expect("failed to open executable file");

        let mut stat = Stat::default();
        vfs().stat(&file, &mut stat).expect("failed to stat executable file");

        let size = stat.size as usize;
        let addr = vmm().allocate_file_backed_vm_object(
            format!("executable '{}' (len={})", executable_file, size),
            file,
            0,
            MapAt::Anywhere,
            size,
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
        ).expect("failed to create file-backed vm object for executable");
        unsafe { from_raw_parts(addr.as_ptr::<u8>(), size) }
    };

    let mut loader = ElfLoader::default();
    let elf = ElfBinary::new(elf_data).unwrap();
    elf.load(&mut loader).unwrap();
    let image = loader.into_inner();
    let entry = unsafe { image.as_ptr().add(elf.entry_point() as usize) };
    let entry_fn = unsafe { transmute::<*const u8, extern "C" fn()>(entry) };

    entry_fn();
}

impl Process {
    pub fn spawn_from_executable(
        parent: &Process,
        path: impl AsRef<Path>,
        priority: Priority,
        uid: RealUserId,
        gid: RealGroupId,
    ) -> Self {
        let path = path.as_ref();

        let proc = Self::create_user(parent, Some(path.to_owned()), path.to_string(), uid, gid);
        spawn_thread("main", &proc, priority, trampoline);

        proc
    }

    pub fn create_kernel(address_space: AddressSpace) -> Self {
        static ALREADY_CALLED: AtomicBool = AtomicBool::new(false);
        if ALREADY_CALLED.swap(true, Relaxed) {
            panic!("kernel process already created");
        }

        let cr3_value = address_space.cr3_value();
        let address_space = Arc::new(RwLock::new(address_space));
        let virtual_memory_manager = Arc::new(unsafe {
            VirtualMemoryManager::new(VirtAddr::new(0x1111_1111_0000), Size::TiB(100).bytes())
        });
        let name = "kernel".to_string();
        let pid = ProcessId::new();
        assert_eq!(0, pid.0, "kernel process must have pid 0");
        let next_fd = Default::default();
        let open_fds = Default::default();
        let attributes = Arc::new(RwLock::new(Attributes {
            pgid: 0.into(),
            euid: 0.into(),
            egid: 0.into(),
            uid: 0.into(),
            gid: 0.into(),
            suid: 0.into(),
            sgid: 0.into(),
        }));

        let res = Self {
            cr3_value,
            address_space,
            virtual_memory_manager,
            name,
            pid,
            next_fd,
            open_fds,
            attributes,
            executable_file: None,
        };
        process_tree().write().set_root(res.clone());
        res
    }

    pub fn create_user(
        parent: &Process,
        executable_file: Option<OwnedPath>,
        name: impl Into<String>,
        uid: RealUserId,
        gid: RealGroupId,
    ) -> Self {
        let address_space = AddressSpace::allocate_new();
        let cr3_value = address_space.cr3_value();
        let vmm = unsafe {
            VirtualMemoryManager::new(VirtAddr::new(0x1111_1111_0000), Size::TiB(100).bytes())
        };

        let name = name.into();
        let pid = ProcessId::new();
        let attributes = Arc::new(RwLock::new(Attributes {
            pgid: 0.into(), // TODO: process group ids
            euid: <RealUserId as Into<u32>>::into(uid).into(),
            egid: <RealGroupId as Into<u32>>::into(gid).into(),
            uid,
            gid,
            suid: <RealUserId as Into<u32>>::into(uid).into(),
            sgid: <RealGroupId as Into<u32>>::into(gid).into(),
        }));

        let res = Self {
            cr3_value,
            address_space: Arc::new(RwLock::new(address_space)),
            virtual_memory_manager: Arc::new(vmm),
            name,
            pid,
            next_fd: Arc::new(Default::default()),
            open_fds: Arc::new(Default::default()),
            attributes,
            executable_file,
        };
        process_tree().write().insert_process(parent.clone(), res.clone());
        res
    }

    pub fn pid(&self) -> &ProcessId {
        &self.pid
    }

    pub fn attributes(&self) -> RwLockReadGuard<Attributes> {
        self.attributes.read()
    }

    pub fn attributes_mut(&self) -> RwLockWriteGuard<Attributes> {
        self.attributes.write()
    }

    pub fn vmm(&self) -> &VirtualMemoryManager {
        &self.virtual_memory_manager
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

    pub fn open_file<P>(&self, path: P) -> Result<Fileno, VfsError>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let node = vfs().open(path)?;
        Ok(self.get_fileno_for(node))
    }

    pub fn allocate_fileno(&self) -> Fileno {
        self.next_fd.next()
    }

    pub fn get_fileno_for(&self, node: VfsNode) -> Fileno {
        let fd = self.allocate_fileno();
        self.open_fds.write().insert(fd, FileDescriptor::new(node));
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

    pub fn stat(&self, fd: Fileno, stat: &mut Stat) -> Result<(), VfsError> {
        let guard = self.open_fds().read();
        let fd = match guard.get(&fd) {
            Some(fd) => fd,
            None => return Err(VfsError::HandleClosed),
        };
        vfs().stat(fd.node(), stat)
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
