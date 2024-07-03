use alloc::vec::Vec;

use derive_more::Constructor;
use x86_64::instructions::interrupts;
use x86_64::structures::paging::{PageSize, PhysFrame, Size4KiB};

use crate::mem::physical::PhysicalMemoryManager;
use crate::mem::virt::AllocationError;

#[derive(Debug, Constructor)]
pub struct PmObject {
    allocation_strategy: PhysicalAllocationStrategy,
    phys_frames: Vec<PhysFrame>,
    should_deallocate_physical_memory_on_drop: bool,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PhysicalAllocationStrategy {
    AllocateNow,
    AllocateOnAccess,
}

impl PmObject {
    pub fn create(
        size: usize,
        allocation_strategy: PhysicalAllocationStrategy,
    ) -> Result<Self, AllocationError> {
        let phys_frames = match allocation_strategy {
            PhysicalAllocationStrategy::AllocateOnAccess => Vec::new(),
            PhysicalAllocationStrategy::AllocateNow => {
                let num_frames = size.div_ceil(Size4KiB::SIZE as usize);
                let mut res = Vec::with_capacity(num_frames);
                let mut guard = PhysicalMemoryManager::lock();
                for _ in 0..num_frames {
                    let next_frame = guard.allocate_frame().ok_or(AllocationError::OutOfMemory);
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
                res
            }
        };

        Ok(Self {
            allocation_strategy,
            phys_frames,
            should_deallocate_physical_memory_on_drop: true, // in this constructor, we allocated the memory, so we have to free it
        })
    }

    pub fn phys_frames(&self) -> &[PhysFrame] {
        &self.phys_frames
    }

    pub fn allocation_strategy(&self) -> PhysicalAllocationStrategy {
        self.allocation_strategy
    }

    pub fn add_phys_frame(&mut self, frame: PhysFrame) {
        self.phys_frames.push(frame);
    }
}

impl Drop for PmObject {
    fn drop(&mut self) {
        assert!(
            interrupts::are_enabled(),
            "interrupts must be enabled when dropping a pmobject"
        );
        deallocate_pm_object(self)
    }
}

fn deallocate_pm_object(pm_object: &PmObject) {
    if !pm_object.should_deallocate_physical_memory_on_drop {
        return;
    }
    
    let mut guard = PhysicalMemoryManager::lock();
    for frame in &pm_object.phys_frames {
        guard.deallocate_frame(*frame);
    }
}
