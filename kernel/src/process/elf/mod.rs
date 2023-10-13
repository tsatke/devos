use alloc::vec::Vec;
use elfloader::{ElfLoaderErr, Flags, LoadableHeaders, RelocationEntry, VAddr};

#[derive(Debug, Default)]
pub struct ElfLoader {
    data: Vec<u8>,
}

impl ElfLoader {
    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }
}

impl elfloader::ElfLoader for ElfLoader {
    fn allocate(&mut self, load_headers: LoadableHeaders) -> Result<(), ElfLoaderErr> {
        for header in load_headers {
            let required_size = header.virtual_addr() as usize + header.mem_size() as usize;
            if self.data.len() < required_size {
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

    fn relocate(&mut self, _entry: RelocationEntry) -> Result<(), ElfLoaderErr> {
        todo!("implement relocation")
    }
}
