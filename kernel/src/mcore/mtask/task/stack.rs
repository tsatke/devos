use crate::mem::address_space::AddressSpace;
use crate::mem::phys::PhysicalMemory;
use crate::mem::virt::{OwnedSegment, VirtualMemory};
use core::ffi::c_void;
use core::fmt::{Debug, Formatter};
use core::slice::from_raw_parts_mut;
use thiserror::Error;
use x86_64::registers::rflags::RFlags;
use x86_64::structures::paging::{PageTableFlags, Size4KiB};
use x86_64::VirtAddr;

#[derive(Debug, Copy, Clone, Error)]
pub enum StackAllocationError {
    #[error("out of virtual memory")]
    OutOfVirtualMemory,
    #[error("out of physical memory")]
    OutOfPhysicalMemory,
}

pub struct Stack {
    segment: OwnedSegment,
    rsp: usize,
}

impl Debug for Stack {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Stack")
            .field("segment", &self.segment)
            .finish_non_exhaustive()
    }
}

impl Drop for Stack {
    fn drop(&mut self) {
        let address_space = AddressSpace::kernel();
        address_space.unmap_range::<Size4KiB>(&*self.segment, PhysicalMemory::deallocate_frame);
    }
}

impl Stack {
    pub fn allocate(
        pages: usize,
        entry_point: extern "C" fn(*mut c_void),
        arg: *mut c_void,
        exit_fn: extern "C" fn(),
    ) -> Result<Self, StackAllocationError> {
        let segment =
            VirtualMemory::reserve(pages).ok_or(StackAllocationError::OutOfVirtualMemory)?;

        // we can use the address space since the segment is in higher half, which is the same
        // for all address spaces
        let address_space = AddressSpace::kernel();
        address_space
            .map_range::<Size4KiB>(
                &*segment,
                PhysicalMemory::allocate_frames_non_contiguous(),
                // FIXME: must be user accessible for user tasks, but can only be user accessible if in lower half, otherwise it can be modified by unrelated tasks/processes
                PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
            )
            .map_err(|_| StackAllocationError::OutOfPhysicalMemory)?;

        // set up stack
        let entry_point = entry_point as *const () as *const usize;
        let stack =
            unsafe { from_raw_parts_mut(segment.start.as_mut_ptr::<u8>(), segment.len as usize) };
        stack.fill(0xCD);

        let mut writer = StackWriter::new(stack);
        writer.push(0xDEAD_BEEF_0BAD_F00D_u64); // marker at stack bottom
        debug_assert_eq!(size_of_val(&exit_fn), size_of::<u64>());
        writer.push(exit_fn);
        let rsp = writer.offset - size_of::<Registers>();
        writer.push(Registers {
            rsp,
            rbp: rsp,
            rdi: arg as usize,
            rip: entry_point as usize,
            rflags: (RFlags::IOPL_LOW | RFlags::INTERRUPT_FLAG).bits() as usize,
            ..Default::default()
        });

        Ok(Self { segment, rsp })
    }
}

impl Stack {
    pub fn initial_rsp(&self) -> VirtAddr {
        self.segment.start + self.rsp as u64
    }
}

#[repr(C, packed)]
#[derive(Debug, Default)]
struct Registers {
    r15: usize,
    r14: usize,
    r13: usize,
    r12: usize,
    r11: usize,
    r10: usize,
    r9: usize,
    r8: usize,
    rdi: usize,
    rsi: usize,
    rsp: usize,
    rbp: usize,
    rdx: usize,
    rcx: usize,
    rbx: usize,
    rax: usize,
    rflags: usize,
    rip: usize,
}

struct StackWriter<'a> {
    stack: &'a mut [u8],
    offset: usize,
}

impl<'a> StackWriter<'a> {
    fn new(stack: &'a mut [u8]) -> Self {
        let len = stack.len();
        Self { stack, offset: len }
    }

    fn push<T>(&mut self, value: T) {
        self.offset = self
            .offset
            .checked_sub(size_of::<T>())
            .expect("should not underflow stack during setup");
        let ptr = self
            .stack
            .as_mut_ptr()
            .wrapping_offset(self.offset as isize) as *mut T;
        unsafe { ptr.write(value) };
    }
}
