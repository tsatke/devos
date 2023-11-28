use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering::Relaxed;

use spin::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use x86_64::VirtAddr;

use crate::io::path::Path;
use crate::io::vfs::{vfs, VfsError, VfsNode};
use crate::mem::virt::VirtualMemoryManager;
use crate::mem::{AddressSpace, Size};
use crate::process::attributes::{Attributes, ProcessId, RealGroupId, RealUserId};
use crate::process::fd::{FileDescriptor, Fileno, FilenoAllocator};
use crate::process::process_tree;

/// A process is a ref counted rwlock over [`ProcessData`], which is the actual data
/// of the process.
#[derive(Clone, Debug)]
pub struct Process {
    cr3_value: usize, // TODO: remove this, read it from the address space (maybe use an atomic to circumvent the locking?)
    address_space: Arc<RwLock<AddressSpace>>,
    virtual_memory_manager: Arc<VirtualMemoryManager>,

    name: String,
    pid: ProcessId,
    next_fd: Arc<FilenoAllocator>,
    open_fds: Arc<RwLock<BTreeMap<Fileno, FileDescriptor>>>,
    attributes: Arc<RwLock<Attributes>>,
}

impl Process {
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
        };
        process_tree().write().set_root(res.clone());
        res
    }

    pub fn create_user(
        parent: Process,
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
        };
        process_tree().write().insert_process(parent, res.clone());
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
