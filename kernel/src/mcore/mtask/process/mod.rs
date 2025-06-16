use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::ffi::c_void;
use core::fmt::{Debug, Formatter};

use conquer_once::spin::OnceCell;
use jiff::Timestamp;
use kernel_elfloader::{ElfFile, ElfLoader};
use kernel_memapi::{Allocation, Location, MemoryApi, UserAccessible};
use kernel_vfs::path::{AbsoluteOwnedPath, AbsolutePath};
use log::debug;
use spin::RwLock;
use thiserror::Error;
use virtual_memory_manager::VirtualMemoryManager;
use x86_64::registers::model_specific::FsBase;
use x86_64::registers::rflags::RFlags;
use x86_64::structures::idt::InterruptStackFrameValue;
use x86_64::VirtAddr;

use crate::file::{vfs, OpenFileDescription};
use crate::mcore::context::ExecutionContext;
use crate::mcore::mtask::process::fd::{FdNum, FileDescriptor, FileDescriptorFlags};
use crate::mcore::mtask::process::tree::process_tree;
use crate::mcore::mtask::task::{Stack, StackAllocationError, StackUserAccessible};
use crate::mem::address_space::AddressSpace;
use crate::mem::memapi::LowerHalfMemoryApi;

pub mod fd;
mod id;
pub use id::*;
use kernel_vfs::Stat;

use crate::time::TimestampExt;

mod tree;

static ROOT_PROCESS: OnceCell<Arc<Process>> = OnceCell::uninit();

pub struct Process {
    pid: ProcessId,
    name: String,

    ppid: RwLock<ProcessId>,

    executable_path: Option<AbsoluteOwnedPath>,
    current_working_directory: RwLock<AbsoluteOwnedPath>,

    address_space: Option<AddressSpace>,
    lower_half_memory: Arc<RwLock<VirtualMemoryManager>>,

    file_descriptors: RwLock<BTreeMap<FdNum, FileDescriptor>>,
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

extern "C" fn trampoline(_arg: *mut c_void) {
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
    let stat = {
        let mut stat = Stat::default();
        node.stat(&mut stat)
            .expect("should be able to stat executable");
        stat
    };

    let start = Timestamp::now();
    let mut data = Vec::with_capacity(stat.size);
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
    let stop = Timestamp::now();
    debug!(
        "read {}KiB in {:?}",
        data.len() / 1024,
        stop.since(start).unwrap(),
    );

    let mut memapi = LowerHalfMemoryApi::new(current_process.clone());

    let elf_file = ElfFile::try_parse(&data).expect("should be able to parse elf binary");
    let elf_image = ElfLoader::new(memapi.clone())
        .load(elf_file)
        .expect("should be able to load elf file");

    if let Some(master_tls) = elf_image.tls_allocation() {
        let mut tls_alloc = memapi
            .allocate(Location::Anywhere, master_tls.layout(), UserAccessible::Yes)
            .expect("should be able to allocate TLS data");

        let slice = tls_alloc.as_mut();
        slice.copy_from_slice(master_tls.as_ref());

        FsBase::write(tls_alloc.start());

        {
            let mut guard = current_task.tls().write();
            assert!(guard.is_none(), "TLS should not exist yet");
            *guard = Some(tls_alloc);
        }
    }

    let ustack = Stack::allocate_plain(
        256,
        &current_process.vmm(),
        StackUserAccessible::Yes,
        current_process.address_space(),
    )
    .expect("should be able to allocate userspace stack");
    let ustack_rsp = ustack.initial_rsp();
    {
        let mut ustack_guard = current_task.ustack().write();
        assert!(ustack_guard.is_none(), "ustack should not exist yet");
        *ustack_guard = Some(ustack);
    }
    assert!(ustack_rsp.is_aligned(16_u64));

    let sel = ctx.selectors();

    let code_ptr = elf_file.entry(); // TODO: this needs to be computed when the elf file is relocatable

    debug!("stack_ptr: {:p}", ustack_rsp.as_ptr::<u8>());
    debug!("code_ptr: {:p}", code_ptr as *const u8);

    {
        let mut guard = current_process.file_descriptors.write();

        let devnull = vfs()
            .read()
            .open(AbsolutePath::try_new("/dev/null").unwrap())
            .expect("should be able to open /dev/null");
        let devnull_ofd = Arc::new(OpenFileDescription::from(devnull));
        guard.insert(
            0.into(),
            FileDescriptor::new(0.into(), FileDescriptorFlags::empty(), devnull_ofd.clone()),
        );

        let devserial = vfs()
            .read()
            .open(AbsolutePath::try_new("/dev/serial").unwrap())
            .expect("should be able to open /dev/serial");
        let devserial_ofd = Arc::new(OpenFileDescription::from(devserial));
        guard.insert(
            1.into(),
            FileDescriptor::new(
                1.into(),
                FileDescriptorFlags::empty(),
                devserial_ofd.clone(),
            ),
        );
        guard.insert(
            2.into(),
            FileDescriptor::new(
                2.into(),
                FileDescriptorFlags::empty(),
                devserial_ofd.clone(),
            ),
        );
    }

    let isfv = InterruptStackFrameValue::new(
        VirtAddr::new(code_ptr as u64),
        sel.user_code,
        RFlags::INTERRUPT_FLAG,
        ustack_rsp,
        sel.user_data,
    );
    unsafe { isfv.iretq() };
}
