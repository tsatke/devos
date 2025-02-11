use alloc::borrow::ToOwned;
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use core::ffi::c_void;
use core::ptr;
use core::slice::from_raw_parts;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering::{Relaxed, Release};

use elfloader::ElfBinary;
use log::trace;
use spin::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use x86_64::structures::paging::PageTableFlags;
use x86_64::VirtAddr;

use kernel_api::syscall::Stat;
pub use scheduler::*;
pub use tree::*;

use crate::io::path::{OwnedPath, Path};
use crate::io::vfs::{vfs, VfsError, VfsNode};
use crate::mem::virt::{MapAt, VirtualMemoryManager};
use crate::mem::{AddressSpace, Size};
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
    let current_thread = unsafe { Thread::kernel_thread(&root_process) };
    process_tree()
        .write()
        .add_thread(root_process.pid(), current_thread.id());

    scheduler::init(current_thread);
}

pub fn current() -> &'static Arc<Process> {
    current_thread().process()
}

pub fn vmm() -> &'static VirtualMemoryManager {
    current().vmm()
}

pub fn current_thread() -> &'static Thread {
    unsafe { scheduler() }.current_thread()
}

pub fn spawn_thread_in_current_process(
    name: impl Into<String>,
    priority: Priority,
    func: extern "C" fn(*mut c_void),
    arg: *mut c_void,
) {
    spawn_thread(name, current(), priority, func, arg)
}

pub fn spawn_thread(
    name: impl Into<String>,
    process: &Arc<Process>,
    priority: Priority,
    func: extern "C" fn(*mut c_void),
    arg: *mut c_void,
) {
    let thread = Thread::new_ready(process, name, priority, func, arg);
    debug_assert_eq!(thread.state(), State::Ready);
    spawn(thread)
}

pub fn change_thread_priority(priority: Priority) {
    unsafe { change_current_thread_prio(priority) }
}

pub fn exit_thread() -> ! {
    unsafe { exit_current_thread() }
}

#[derive(Debug)]
pub struct Process {
    // TODO: remove this, read it from the address space (maybe use an atomic to circumvent the locking?)
    cr3_value: usize,
    address_space: RwLock<AddressSpace>,
    virtual_memory_manager: VirtualMemoryManager,

    name: String,
    pid: ProcessId,
    should_terminate: AtomicBool,
    next_fd: FilenoAllocator,
    open_fds: RwLock<BTreeMap<Fileno, FileDescriptor>>,
    attributes: RwLock<Attributes>,

    executable_file: Option<OwnedPath>,
}

extern "C" fn trampoline(_: *mut c_void) {
    let proc = current();
    if proc.executable_file.is_none() {
        panic!("trampoline called for a process without an executable file");
    }
    let executable_file = proc.executable_file.as_ref().unwrap().as_path();

    let elf_data = {
        let file = vfs()
            .open(executable_file)
            .expect("failed to open executable file");

        let mut stat = Stat::default();
        vfs()
            .stat(&file, &mut stat)
            .expect("failed to stat executable file");

        let size = stat.size as usize;
        let addr = vmm()
            .allocate_file_backed_vm_object(
                format!("executable '{}' (len={})", executable_file, size),
                file,
                0,
                MapAt::Anywhere,
                size,
                PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
            )
            .expect("failed to create file-backed vm object for executable");
        unsafe { from_raw_parts(addr.as_ptr::<u8>(), size) }
    };

    let mut loader = ElfLoader::default();
    let elf = ElfBinary::new(elf_data).unwrap();
    elf.load(&mut loader).unwrap();
    let image = loader.into_inner();
    let code_ptr = unsafe { image.as_ptr().add(elf.entry_point() as usize) };

    let entry_fn: extern "C" fn() = unsafe { core::mem::transmute(code_ptr) };

    entry_fn();

    unreachable!("entry function returned, but must call sys_exit instead");

    // TODO: I guess before we can jump to entry_fn in usermode, we need to make sure that the code and stack are actually in user space instead of the kernel heap.

    // let stack_ptr = read_rsp();
    //
    // let (cs, ds) = {
    //     // set ds and tss, return cs and ds
    //
    //     let (mut cs, mut ds) = (GDT.1.user_code_selector, GDT.1.user_data_selector);
    //     cs.0 |= PrivilegeLevel::Ring3 as u16;
    //     ds.0 |= PrivilegeLevel::Ring3 as u16;
    //     unsafe { DS::set_reg(ds) };
    //     (cs.0, ds.0)
    // };
    //
    // tlb::flush_all();
    //
    // unsafe {
    //     asm!(
    //     "push {stack_segment:r}",
    //     "push {stack_ptr}",
    //     "push 0x200", // rflags with only interrupts enabled
    //     "push {code_segment:r}",
    //     "push {code_ptr}",
    //     "iretq",
    //     stack_segment = in(reg) ds,
    //     stack_ptr = in(reg) stack_ptr,
    //     code_segment = in(reg) cs,
    //     code_ptr = in(reg) code_ptr,
    //     );
    // }
}

// fn read_rsp() -> usize {
//     let rsp: usize;
//     unsafe { asm!("mov {}, rsp", out(reg) rsp) };
//     rsp
// }

impl Process {
    pub fn spawn_from_executable(
        parent: &Arc<Process>,
        path: impl AsRef<Path>,
        priority: Priority,
        uid: RealUserId,
        gid: RealGroupId,
    ) -> Arc<Self> {
        let path = path.as_ref();

        let proc = Self::create_user(parent, Some(path.to_owned()), path.to_string(), uid, gid);
        spawn_thread("main", &proc, priority, trampoline, ptr::null_mut());

        proc
    }

    pub fn create_kernel(address_space: AddressSpace) -> Arc<Self> {
        static ALREADY_CALLED: AtomicBool = AtomicBool::new(false);
        if ALREADY_CALLED.swap(true, Relaxed) {
            panic!("kernel process already created");
        }

        let cr3_value = address_space.cr3_value();
        let address_space = RwLock::new(address_space);
        let virtual_memory_manager = unsafe {
            VirtualMemoryManager::new(VirtAddr::new(0x1111_1111_0000), Size::TiB(100).bytes())
        };
        let name = "kernel".to_string();
        let pid = ProcessId::new();
        assert_eq!(0, pid.0, "kernel process must have pid 0");
        let next_fd = Default::default();
        let open_fds = Default::default();
        let attributes = RwLock::new(Attributes {
            pgid: 0.into(),
            euid: 0.into(),
            egid: 0.into(),
            uid: 0.into(),
            gid: 0.into(),
            suid: 0.into(),
            sgid: 0.into(),
        });

        let res = Arc::new(Self {
            cr3_value,
            address_space,
            virtual_memory_manager,
            name,
            pid,
            should_terminate: AtomicBool::new(false),
            next_fd,
            open_fds,
            attributes,
            executable_file: None,
        });
        process_tree().write().set_root(res.clone());
        res
    }

    pub fn create_user(
        parent: &Process,
        executable_file: Option<OwnedPath>,
        name: impl Into<String>,
        uid: RealUserId,
        gid: RealGroupId,
    ) -> Arc<Self> {
        let address_space = AddressSpace::allocate_new();
        let cr3_value = address_space.cr3_value();
        let vmm = unsafe {
            VirtualMemoryManager::new(VirtAddr::new(0x1111_1111_0000), Size::TiB(100).bytes())
        };

        let name = name.into();
        let pid = ProcessId::new();
        let attributes = RwLock::new(Attributes {
            pgid: 0.into(), // TODO: process group ids
            euid: u32::from(uid).into(),
            egid: u32::from(gid).into(),
            uid,
            gid,
            suid: u32::from(uid).into(),
            sgid: u32::from(gid).into(),
        });

        let res = Arc::new(Self {
            cr3_value,
            address_space: RwLock::new(address_space),
            virtual_memory_manager: vmm,
            name,
            pid,
            should_terminate: AtomicBool::new(false),
            next_fd: Default::default(),
            open_fds: Default::default(),
            attributes,
            executable_file,
        });
        process_tree()
            .write()
            .insert_process(*parent.pid(), res.clone());
        res
    }

    pub fn terminate(&self) {
        assert!(self.address_space.read().is_active());

        trace!("terminating process {} ({})", self.pid, self.name);

        // drop open file descriptors - drop must take care of flushing
        self.open_fds().write().clear();

        // drop vm objects - drop takes care of unmapping
        self.vmm().vm_objects().write().clear();

        self.should_terminate.store(true, Release);
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

impl Drop for Process {
    fn drop(&mut self) {
        assert_eq!(
            0,
            self.open_fds().read().len(),
            "open file descriptors must be flushed and closed before dropping the process"
        );
        assert_eq!(
            0,
            self.vmm().vm_objects().read().len(),
            "vm objects must be unmapped before dropping the process"
        );
    }
}
