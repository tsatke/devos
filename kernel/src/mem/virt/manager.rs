use alloc::collections::BTreeSet;
use core::ops::{Deref, DerefMut};

use derive_more::Constructor;
use spin::RwLock;
use x86_64::VirtAddr;

use crate::mem::virt::heap::heap_initialized;

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
    NoMoreSpace, // not out of memory, because the memory manager has its own boundaries
}

#[derive(Debug)]
pub struct VirtualMemoryManager {
    mem_start: VirtAddr,
    mem_size: usize,
    inner: RwLock<Intervals>,
}

impl VirtualMemoryManager {
    pub fn new(mem_start: VirtAddr, mem_size: usize) -> Self {
        assert!(heap_initialized());
        Self {
            mem_start,
            mem_size,
            inner: RwLock::new(Intervals(BTreeSet::default())),
        }
    }

    pub fn reserve(&self, size: usize) -> Result<Interval, VmmError> {
        let mut interval = Interval::new(self.mem_start, size);
        let mut guard = self.inner.write();
        while let Some(existing) = guard.find_overlapping_element(interval.start, interval.size) {
            interval.start = existing.start + existing.size;
        }
        if interval.start + interval.size > self.mem_start + self.mem_size {
            return Err(VmmError::NoMoreSpace);
        }

        guard.insert(interval);

        Ok(interval)
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

#[derive(Debug, Clone, Eq, PartialEq)]
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
        let vmm = VirtualMemoryManager::new(VirtAddr::new(0x0), 0x10000);
        let interval = vmm.reserve(0xf000).unwrap();
        // depending on the implementation, these may change
        assert_eq!(interval.start, VirtAddr::new(0x0));
        assert_eq!(interval.size, 0xf000);

        // this allocation is too large
        assert_eq!(VmmError::NoMoreSpace, vmm.reserve(0x2000).unwrap_err());

        // we still need to be able to allocate after a failed attempt
        let interval = vmm.reserve(0x1000).unwrap();
        assert_eq!(interval.start, VirtAddr::new(0xf000));
        assert_eq!(interval.size, 0x1000);
    }

    #[kernel_test]
    fn test_would_overlap_with_existing() {
        let vmm = VirtualMemoryManager::new(VirtAddr::new(0x0), 0x10000);
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
