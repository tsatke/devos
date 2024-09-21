use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use core::error::Error;
use core::fmt::{Debug, Formatter};
use core::ops::{Deref, DerefMut};

use derive_more::{Constructor, Display};
use spin::RwLock;
use x86_64::structures::paging::{PageSize, PageTableFlags, PhysFrame, Size4KiB};
use x86_64::VirtAddr;

use kernel_api::syscall::Errno;

use crate::io::vfs::VfsNode;
use crate::mem::physical::PhysicalMemoryManager;
use crate::mem::virt::heap::heap_initialized;
use crate::mem::virt::{
    FileBackedVmObject, MemoryBackedVmObject, PhysicalAllocationStrategy, PmObject, VmObject,
};

/// Represents a memory range in a given address space with ownership. Dropping an instance
/// makes the represented memory range available for reallocation. The `OwnedInterval`
/// is used to manage memory ranges that, when dropped, should be returned to the available
/// pool of memory by the virtual memory manager this interval was obtained by.
///
/// Note that the `OwnedInterval` always refers to the virtual memory manager that it was
/// obtained from.
pub struct OwnedInterval<'a> {
    interval: Interval,
    vmm: &'a VirtualMemoryManager,
}

impl Debug for OwnedInterval<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("OwnedInterval")
            .field("interval", &self.interval)
            .finish()
    }
}

impl OwnedInterval<'_> {
    /// Prevents the automatic deallocation of the memory range represented by this struct
    /// upon dropping. The memory range remains allocated indefinitely. This method is
    /// intended for cases where the memory should not be returned to the pool for reallocation,
    /// such as with structures that are managed outside the kernel, such as the level-4 page table
    /// that is effectively owned by the `CR3` register.
    pub fn leak(self) -> Interval {
        core::mem::ManuallyDrop::new(self).interval
    }
}

impl Deref for OwnedInterval<'_> {
    type Target = Interval;

    fn deref(&self) -> &Self::Target {
        &self.interval
    }
}

impl Drop for OwnedInterval<'_> {
    fn drop(&mut self) {
        let interval = self.interval;
        let _ = self.vmm.release(interval); // FIXME: what happens when this returns false?
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Constructor)]
pub struct Interval {
    start: VirtAddr,
    size: usize,
}

impl Interval {
    pub fn start(&self) -> VirtAddr {
        self.start
    }

    pub fn size(&self) -> usize {
        self.size
    }
}

#[derive(Display, Debug, Copy, Clone, Eq, PartialEq)]
pub enum VmmError {
    #[display = "requested memory is already allocated"]
    AlreadyAllocated,
    #[display = "out of memory"]
    OutOfMemory,
}

impl Error for VmmError {}

impl From<VmmError> for Errno {
    fn from(value: VmmError) -> Self {
        match value {
            VmmError::AlreadyAllocated | VmmError::OutOfMemory => Errno::ENOMEM,
        }
    }
}

#[derive(Debug)]
pub struct VirtualMemoryManager {
    mem_start: VirtAddr,
    mem_size: usize,
    inner: RwLock<Intervals>,
    vm_objects: RwLock<BTreeMap<VirtAddr, Box<dyn VmObject>>>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum MapAt {
    Fixed(VirtAddr),
    Anywhere,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum AllocationStrategy<'a> {
    AllocateOnAccess,
    AllocateNow,
    MapNow(&'a [PhysFrame]),
}

impl VirtualMemoryManager {
    /// # Safety
    /// The caller must ensure that the memory from `mem_start` to `mem_start + mem_size` is
    /// unused, unmapped and is effectively owned by this `VirtualMemoryManager`.
    /// `mem_start` must be page aligned.
    pub unsafe fn new(mem_start: VirtAddr, mem_size: usize) -> Self {
        assert!(mem_start.is_aligned(Size4KiB::SIZE));
        assert!(heap_initialized());
        Self {
            mem_start,
            mem_size,
            inner: Default::default(),
            vm_objects: Default::default(),
        }
    }

    pub fn allocate_memory_backed_vmobject(
        &'static self,
        name: String,
        addr: MapAt,
        size: usize,
        allocation_strategy: AllocationStrategy,
        flags: PageTableFlags,
    ) -> Result<VirtAddr, VmmError> {
        let vmo = self.create_memory_backed_vmo(name, addr, size, allocation_strategy, flags)?;

        let addr = vmo.addr();
        self.vm_objects.write().insert(addr, Box::new(vmo));

        Ok(addr)
    }

    pub fn allocate_file_backed_vm_object(
        &'static self,
        name: String,
        node: VfsNode,
        offset: usize,
        addr: MapAt,
        size: usize,
        flags: PageTableFlags,
    ) -> Result<VirtAddr, VmmError> {
        let memory_backed = self.create_memory_backed_vmo(
            name,
            addr,
            size,
            AllocationStrategy::AllocateOnAccess,
            flags,
        )?;
        let vmo = FileBackedVmObject::new(node, offset, memory_backed);

        let addr = vmo.addr();
        self.vm_objects.write().insert(addr, Box::new(vmo));

        Ok(addr)
    }

    fn create_memory_backed_vmo(
        &'static self,
        name: String,
        addr: MapAt,
        size: usize,
        allocation_strategy: AllocationStrategy,
        flags: PageTableFlags,
    ) -> Result<MemoryBackedVmObject, VmmError> {
        let interval = self.resolve_map_at(addr, size)?;

        let (physical_memory, should_map, should_zero) = match allocation_strategy {
            AllocationStrategy::AllocateOnAccess => (vec![], false, false),
            AllocationStrategy::AllocateNow => {
                let num_frames = size.div_ceil(Size4KiB::SIZE as usize);
                (allocate_phys_frames(num_frames)?, true, true)
            }
            AllocationStrategy::MapNow(frames) => (frames.to_vec(), true, false),
        };

        let vmo = MemoryBackedVmObject::new(
            name,
            Arc::new(RwLock::new(PmObject::new(
                PhysicalAllocationStrategy::AllocateOnAccess,
                physical_memory,
                // don't deallocate if we get handed the physical frames that are used
                !matches!(allocation_strategy, AllocationStrategy::MapNow(_)),
            ))),
            interval,
            flags,
        );

        if should_map {
            vmo.map_pages()?;
        }
        if should_zero {
            unsafe { vmo.as_slice_mut().fill(0) };
        }
        Ok(vmo)
    }

    fn resolve_map_at(&self, addr: MapAt, size: usize) -> Result<OwnedInterval, VmmError> {
        Ok(match addr {
            MapAt::Fixed(addr) => {
                let interval = Interval::new(addr, size);
                self.mark_as_reserved(interval)?
            }
            MapAt::Anywhere => self.reserve(size)?,
        })
    }

    pub fn vm_objects(&self) -> &RwLock<BTreeMap<VirtAddr, Box<dyn VmObject>>> {
        &self.vm_objects
    }

    pub fn reserve(&self, size: usize) -> Result<OwnedInterval, VmmError> {
        let size = align_up_to::<Size4KiB>(size);
        let mut interval = Interval::new(self.mem_start, size);
        let mut guard = self.inner.write();
        while let Some(existing) = guard.find_overlapping_element(interval.start, interval.size) {
            interval.start = existing.start + existing.size;
        }
        if interval.start + interval.size > self.mem_start + self.mem_size {
            return Err(VmmError::OutOfMemory);
        }

        guard.insert(interval);

        let owned = OwnedInterval {
            interval,
            vmm: self,
        };
        Ok(owned)
    }

    pub fn release(&self, interval: Interval) -> bool {
        let mut guard = self.inner.write();
        guard.remove(&interval)
    }

    pub fn mark_as_reserved(&self, interval: Interval) -> Result<OwnedInterval, VmmError> {
        let mut guard = self.inner.write();
        if guard
            .find_overlapping_element(interval.start, interval.size)
            .is_some()
        {
            return Err(VmmError::AlreadyAllocated);
        }
        guard.insert(interval);

        let owned = OwnedInterval {
            interval,
            vmm: self,
        };
        Ok(owned)
    }
}

fn allocate_phys_frames(num_frames: usize) -> Result<Vec<PhysFrame>, VmmError> {
    let mut res = Vec::with_capacity(num_frames);
    let mut guard = PhysicalMemoryManager::lock();
    for _ in 0..num_frames {
        let next_frame = guard.allocate_frame().ok_or(VmmError::OutOfMemory);
        match next_frame {
            Ok(frame) => res.push(frame),
            Err(e) => {
                // if allocation fails, deallocate the frames we already allocated
                for frame in res {
                    guard.deallocate_frame(frame);
                }
                return Err(e);
            }
        }
    }
    Ok(res)
}

#[derive(Default, Debug, Clone, Eq, PartialEq)]
struct Intervals(BTreeSet<Interval>); // TODO: there's probably a better data structure for this than a btreeset

impl Deref for Intervals {
    type Target = BTreeSet<Interval>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Intervals {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Intervals {
    fn find_overlapping_element(&self, start: VirtAddr, size: usize) -> Option<Interval> {
        self.iter()
            .find(|existing| {
                start < existing.start + existing.size && existing.start < start + size
            })
            .cloned()
    }
}

fn align_up_to<P: PageSize>(v: usize) -> usize {
    let v = v as u64;
    let align_mask = P::SIZE - 1;
    (if v & align_mask == 0 {
        v // already aligned
    } else {
        (v | align_mask).checked_add(1).unwrap()
    }) as usize
}

#[cfg(feature = "kernel_test")]
mod tests {
    use x86_64::VirtAddr;

    use kernel_test_framework::kernel_test;

    use crate::mem::virt::{Interval, VirtualMemoryManager, VmmError};

    #[kernel_test]
    fn test_allocate() {
        let vmm = unsafe { VirtualMemoryManager::new(VirtAddr::new(0x0), 0x10000) };
        let interval = vmm.reserve(0xf000).unwrap();
        // depending on the implementation, these may change
        assert_eq!(interval.start, VirtAddr::new(0x0));
        assert_eq!(interval.size, 0xf000);

        // this allocation is too large
        assert_eq!(VmmError::OutOfMemory, vmm.reserve(0x2000).unwrap_err());

        // we still need to be able to allocate after a failed attempt
        let interval = vmm.reserve(0x1000).unwrap();
        assert_eq!(interval.start, VirtAddr::new(0xf000));
        assert_eq!(interval.size, 0x1000);
    }

    #[kernel_test]
    fn test_would_overlap_with_existing() {
        let vmm = unsafe { VirtualMemoryManager::new(VirtAddr::new(0x0), 0x10000) };
        let owned = vmm
            .mark_as_reserved(Interval::new(VirtAddr::new(0x2000), 0x1000))
            .unwrap();
        {
            let guard = vmm.inner.read();
            assert!(guard
                .find_overlapping_element(VirtAddr::new(0x1000), 0x1000)
                .is_none());
            assert!(guard
                .find_overlapping_element(VirtAddr::new(0x1000), 0x1001)
                .is_some());
            assert!(guard
                .find_overlapping_element(VirtAddr::new(0x1000), 0x3000)
                .is_some());
            assert!(guard
                .find_overlapping_element(VirtAddr::new(0x2a00), 0x2f00)
                .is_some());
        }
        drop(owned);
        {
            let guard = vmm.inner.read();
            assert!(guard
                .find_overlapping_element(VirtAddr::new(0x0), 0x10000)
                .is_none());
        }
    }
}
