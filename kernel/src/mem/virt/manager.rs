use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use core::ops::{Deref, DerefMut};

use derive_more::Constructor;
use spin::RwLock;
use x86_64::structures::paging::{PageSize, PageTableFlags, PhysFrame, Size4KiB};
use x86_64::VirtAddr;

use crate::io::vfs::VfsNode;
use crate::mem::physical::PhysicalMemoryManager;
use crate::mem::virt::heap::heap_initialized;
use crate::mem::virt::{
    AllocationStrategy_, FileBackedVmObject, MemoryBackedVmObject, PmObject, VmObject,
};

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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum VmmError {
    AlreadyAllocated,
    OutOfMemory,
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
pub enum AllocationStrategy {
    AllocateOnAccess,
    AllocateNow,
    MapNow(Vec<PhysFrame>),
}

impl VirtualMemoryManager {
    /// # Safety
    /// The caller must ensure that the memory from `mem_start` to `mem_start + mem_size` is
    /// unused, unmapped and is effectively owned by this `VirtualMemoryManager`.
    pub unsafe fn new(mem_start: VirtAddr, mem_size: usize) -> Self {
        assert!(heap_initialized());
        Self {
            mem_start,
            mem_size,
            inner: Default::default(),
            vm_objects: Default::default(),
        }
    }

    pub fn allocate_memory_backed_vmobject(
        &self,
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
        &self,
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
        &self,
        name: String,
        addr: MapAt,
        size: usize,
        allocation_strategy: AllocationStrategy,
        flags: PageTableFlags,
    ) -> Result<MemoryBackedVmObject, VmmError> {
        let interval = match addr {
            MapAt::Fixed(addr) => {
                let interval = Interval::new(addr, size);
                self.mark_as_reserved(interval)?;
                interval
            }
            MapAt::Anywhere => self.reserve(size)?,
        };

        let (physical_memory, should_map, should_zero) = match allocation_strategy {
            AllocationStrategy::AllocateOnAccess => (vec![], false, false),
            AllocationStrategy::AllocateNow => {
                let num_frames = size.div_ceil(Size4KiB::SIZE as usize);
                (allocate_phys_frames(num_frames)?, true, true)
            }
            AllocationStrategy::MapNow(frames) => (frames, true, false),
        };

        let vmo = MemoryBackedVmObject::new(
            name,
            Arc::new(RwLock::new(PmObject::new(
                AllocationStrategy_::AllocateOnAccess,
                physical_memory,
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

    pub fn vm_objects(&self) -> &RwLock<BTreeMap<VirtAddr, Box<dyn VmObject>>> {
        &self.vm_objects
    }

    pub fn reserve(&self, size: usize) -> Result<Interval, VmmError> {
        let mut interval = Interval::new(self.mem_start, size);
        let mut guard = self.inner.write();
        while let Some(existing) = guard.find_overlapping_element(interval.start, interval.size) {
            interval.start = existing.start + existing.size;
        }
        if interval.start + interval.size > self.mem_start + self.mem_size {
            return Err(VmmError::OutOfMemory);
        }

        guard.insert(interval);

        Ok(interval)
    }

    pub fn release(&self, interval: Interval) {
        let mut guard = self.inner.write();
        assert!(guard.remove(&interval));
    }

    pub fn mark_as_reserved(&self, interval: Interval) -> Result<(), VmmError> {
        let mut guard = self.inner.write();
        if guard
            .find_overlapping_element(interval.start, interval.size)
            .is_some()
        {
            return Err(VmmError::AlreadyAllocated);
        }
        guard.insert(interval);
        Ok(())
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
        vmm.mark_as_reserved(Interval::new(VirtAddr::new(0x2000), 0x1000))
            .unwrap();
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
}
