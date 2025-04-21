use crate::mcore::mtask::process::Process;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use elfloader::arch::x86_64::RelocationTypes;
use elfloader::{ElfLoaderErr, Flags, LoadableHeaders, RelocationEntry, RelocationType, VAddr};
use log::debug;

pub struct ElfLoader {
    process: Arc<Process>,
    data: Vec<u8>,
}

impl ElfLoader {
    pub fn new(process: Arc<Process>) -> Self {
        Self {
            process,
            data: vec![],
        }
    }

    pub(crate) fn into_inner(self) -> Vec<u8> {
        self.data
    }
}

impl elfloader::ElfLoader for ElfLoader {
    fn allocate(&mut self, load_headers: LoadableHeaders) -> Result<(), ElfLoaderErr> {
        for header in load_headers {
            let required_size = header.virtual_addr() as usize + header.mem_size() as usize;
            if self.data.len() < required_size {
                debug!("resizing elf image allocation to 0x{required_size:x} bytes");
                self.data.resize(required_size, 0);
            }
        }
        Ok(())
    }

    fn load(&mut self, _flags: Flags, base: VAddr, region: &[u8]) -> Result<(), ElfLoaderErr> {
        // FIXME: properly allocate and respect flags
        let dest = &mut self.data[base as usize..base as usize + region.len()];
        dest.copy_from_slice(region);
        Ok(())
    }

    fn relocate(&mut self, entry: RelocationEntry) -> Result<(), ElfLoaderErr> {
        let typ = match entry.rtype {
            RelocationType::x86_64(v) => v,
            RelocationType::x86(_) | RelocationType::AArch64(_) => {
                return Err(ElfLoaderErr::UnsupportedArchitecture);
            }
        };
        match typ {
            RelocationTypes::R_AMD64_RELATIVE => {
                // *target_addr = (base_address + addend)
                let base_address = self.data.as_ptr() as usize;
                let value = base_address + entry.addend.unwrap() as usize;
                let value_bytes = value.to_ne_bytes();
                let dest = &mut self.data
                    [entry.offset as usize..entry.offset as usize + size_of::<usize>()];
                dest.copy_from_slice(&value_bytes);
            }
            _ => return Err(ElfLoaderErr::UnsupportedRelocationEntry),
        };
        Ok(())
    }
}
