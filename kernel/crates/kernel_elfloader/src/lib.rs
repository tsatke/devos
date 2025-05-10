#![no_std]
extern crate alloc;

mod file;

use alloc::vec::Vec;
pub use file::*;
use thiserror::Error;

pub trait MemoryAllocator {
    type Allocation: Allocation;

    fn allocate(&mut self, size: usize, align: usize) -> Option<Self::Allocation>;
}

/// Describes an allocation created by [`MemoryAllocator`]. An implementation
/// must free the allocation upon drop.
pub trait Allocation: AsRef<[u8]> + AsMut<[u8]> {}

pub struct ElfLoader<'a, A>
where
    A: MemoryAllocator,
{
    allocator: A,
    elf_file: ElfFile<'a>,
    allocations: Vec<A::Allocation>,
}

#[derive(Debug, Eq, PartialEq, Error)]
pub enum LoadElfError {
    #[error("could not allocate memory")]
    AllocationFailed,
    #[error("unsupported file type")]
    UnsupportedFileType(ElfType),
}

impl<'a, A> ElfLoader<'a, A>
where
    A: MemoryAllocator,
{
    pub fn new(allocator: A, elf_file: ElfFile<'a>) -> Self {
        Self {
            allocator,
            elf_file,
            allocations: Vec::new(),
        }
    }

    pub fn load(&mut self) -> Result<(), LoadElfError> {
        self.load_stage1()?;
        todo!()
    }

    fn load_stage1(&mut self) -> Result<(), LoadElfError> {
        for hdr in self
            .elf_file
            .section_headers()
            .filter(|hdr| hdr.typ == SectionHeaderType::NOBITS)
            .filter(|hdr| hdr.size > 0)
            .filter(|hdr| hdr.flags.contains(&SectionHeaderFlags::ALLOC))
        {
            let size = hdr.size;
            let align = hdr.addralign;

            let mut alloc = self
                .allocator
                .allocate(size, align)
                .ok_or(LoadElfError::AllocationFailed)?;
            alloc.as_mut().fill(0);

            self.allocations.push(alloc);
        }

        Ok(())
    }
}
