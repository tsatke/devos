use alloc::vec::Vec;

use x86_64::instructions::interrupts;
use x86_64::structures::paging::{PageSize, PhysFrame, Size4KiB};

use crate::io::vfs::VfsNode;
use crate::mem::physical::PhysicalMemoryManager;
use crate::mem::virt::AllocationError;

#[derive(Debug)]
pub struct PmObject {
    kind: PmObjectKind,
    phys_frames: Vec<PhysFrame>,
}

#[derive(Debug)]
pub enum PmObjectKind {
    Memory(AllocationStrategy),
    File(File),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum AllocationStrategy {
    AllocateNow,
    AllocateOnAccess,
}

#[derive(Debug)]
pub struct File {
    pub node: VfsNode,
    pub offset: usize,
    pub size: usize,
}

impl PmObject {
    pub fn create_memory_backed(
        size: usize,
        allocation_strategy: AllocationStrategy,
    ) -> Result<Self, AllocationError> {
        let phys_frames = match allocation_strategy {
            AllocationStrategy::AllocateOnAccess => Vec::new(),
            AllocationStrategy::AllocateNow => {
                let num_frames = size.div_ceil(Size4KiB::SIZE as usize);
                let mut res = Vec::with_capacity(num_frames);
                let mut guard = PhysicalMemoryManager::lock();
                for _ in 0..num_frames {
                    let next_frame = guard.allocate_frame().ok_or(AllocationError);
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
            kind: PmObjectKind::Memory(allocation_strategy),
            phys_frames,
        })
    }

    pub fn kind(&self) -> &PmObjectKind {
        &self.kind
    }

    pub fn phys_frames(&self) -> &[PhysFrame] {
        &self.phys_frames
    }

    pub fn allocation_strategy(&self) -> AllocationStrategy {
        match self.kind {
            PmObjectKind::Memory(v) => v,
            PmObjectKind::File(_) => AllocationStrategy::AllocateOnAccess,
        }
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
    let mut guard = PhysicalMemoryManager::lock();
    for frame in &pm_object.phys_frames {
        guard.deallocate_frame(*frame);
    }
}

impl PmObjectKind {
    pub fn is_memory_backed(&self) -> bool {
        matches!(self, PmObjectKind::Memory(_))
    }

    pub fn allocation_strategy(&self) -> Option<&AllocationStrategy> {
        match self {
            PmObjectKind::Memory(strategy) => Some(strategy),
            _ => None,
        }
    }

    pub fn is_file_backed(&self) -> bool {
        matches!(self, PmObjectKind::File(_))
    }

    pub fn file(&self) -> Option<&File> {
        match self {
            PmObjectKind::File(file) => Some(file),
            _ => None,
        }
    }
}
