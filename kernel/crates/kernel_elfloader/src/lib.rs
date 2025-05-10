#![no_std]
extern crate alloc;

mod file;

use alloc::collections::BTreeMap;
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
    allocations: BTreeMap<usize, A::Allocation>,
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
            allocations: BTreeMap::new(),
        }
    }

    pub fn load(&mut self) -> Result<(), LoadElfError> {
        for (index, hdr) in self
            .elf_file
            .program_headers()
            .enumerate()
            .filter(|(_, hdr)| [ProgramHeaderType::LOAD, ProgramHeaderType::TLS].contains(&hdr.typ))
        {
            let pdata = self.elf_file.program_data(hdr);

            let mut alloc = self
                .allocator
                .allocate(hdr.memsz, hdr.align)
                .ok_or(LoadElfError::AllocationFailed)?;

            let mut slice = alloc.as_mut();
            slice[..hdr.filesz].copy_from_slice(pdata);
            slice[hdr.filesz..].fill(0);

            self.allocations.insert(index, alloc);
        }

        Ok(())
    }
}
