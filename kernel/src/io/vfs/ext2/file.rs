use crate::io::vfs::ext2::InnerHandle;
use crate::io::vfs::{File, InodeBase, InodeNum, IoError, ReadError, Stat, WriteError};
use crate::serial_println;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use conquer_once::spin::Lazy;
use core::mem::size_of;
use ext2::BlockAddress;
use filesystem::BlockDevice;

pub struct Ext2File<T> {
    inner: InnerHandle<T>,
    file: ext2::RegularFile,
    name: String,
}

impl<T> Ext2File<T>
where
    T: BlockDevice + 'static + Send + Sync,
{
    pub(crate) fn new(inner: InnerHandle<T>, file: ext2::RegularFile, name: String) -> Self {
        Self { inner, file, name }
    }

    /// Takes the index of a block within a file node and determines, in which pointer list to look
    /// for the block address. For example, a block index of 0 (the very first block in a file) is
    /// stored in the direct pointers list. A block index of 12 is the first entry in the single
    /// indirect pointers list.
    fn determine_block_pointer_type(&self, block_index: usize) -> BlockPointerType {
        let block_size = self.inner.read().fs.superblock().block_size();
        let pointers_per_block = (block_size / 4) as usize;
        let hi_single_indirect = pointers_per_block;
        let hi_double_indirect = hi_single_indirect * pointers_per_block;
        let hi_triple_indirect = hi_double_indirect * pointers_per_block;

        let direct_range = 0_usize..12;
        let single_indirect_range = 12..hi_single_indirect;
        let double_indirect_range = hi_single_indirect..hi_double_indirect;
        let triple_indirect_range = hi_double_indirect..hi_triple_indirect;

        match block_index {
            x if direct_range.contains(&x) => BlockPointerType::Direct,
            x if single_indirect_range.contains(&x) => BlockPointerType::SingleIndirect,
            x if double_indirect_range.contains(&x) => BlockPointerType::DoubleIndirect,
            x if triple_indirect_range.contains(&x) => BlockPointerType::TripleIndirect,
            _ => unreachable!("too many blocks"),
        }
    }
}

impl<T> InodeBase for Ext2File<T>
where
    T: filesystem::BlockDevice + Send + Sync,
{
    fn num(&self) -> InodeNum {
        1_u64.into()
    }

    fn name(&self) -> String {
        self.name.clone() // TODO: remove clone
    }

    fn stat(&self) -> Stat {
        Stat {
            ..Default::default()
        }
    }
}

impl<T> File for Ext2File<T>
where
    T: BlockDevice + 'static + Send + Sync,
{
    fn size(&self) -> u64 {
        self.file.len() as u64
    }

    fn truncate(&mut self, _size: u64) -> Result<(), WriteError> {
        todo!()
    }

    fn read_at(&self, offset: u64, buf: &mut dyn AsMut<[u8]>) -> Result<usize, ReadError> {
        let buffer = buf.as_mut();

        let block_size = self.inner.read().fs.superblock().block_size() as usize;

        let start_block = offset as u32 / block_size as u32;
        let end_block = (offset as u32 + buffer.len() as u32) / block_size as u32;
        let relative_offset = offset as usize % block_size;
        let block_count = if buffer.len() % block_size == 0 {
            end_block - start_block
        } else {
            end_block - start_block + 1
        } as usize;

        // read blocks
        let mut data: Vec<u8> = vec![0_u8; block_count * block_size];
        let single_indirect_cache = Lazy::new(|| {
            let pointer_block = self.file.single_indirect_ptr().unwrap();
            let mut pointer_data = vec![0_u8; block_size];
            self.inner
                .read()
                .fs
                .read_block(pointer_block, &mut pointer_data)
                .expect("failed to read single indirect pointer block");
            pointer_data
        });
        let double_indirect_cache = Lazy::new(|| {
            let pointer_block = self.file.double_indirect_ptr().unwrap();
            let mut pointer_data = vec![0_u8; block_size];
            self.inner
                .read()
                .fs
                .read_block(pointer_block, &mut pointer_data)
                .expect("failed to read double indirect pointer block");
            pointer_data
        });

        const SZ: usize = size_of::<u32>();
        for i in 0..block_count {
            let read_block_index = (start_block + i as u32) as usize;
            let block_pointer = match self.determine_block_pointer_type(read_block_index) {
                BlockPointerType::Direct => self.file.direct_ptr(read_block_index),
                BlockPointerType::SingleIndirect => single_indirect_cache
                    .iter()
                    .copied()
                    .array_chunks::<SZ>()
                    .map(u32::from_le_bytes)
                    .map(BlockAddress::new)
                    .nth(read_block_index - 12) // 12 direct pointers, so subtract the 12
                    .unwrap(),
                BlockPointerType::DoubleIndirect => double_indirect_cache
                    .iter()
                    .copied()
                    .array_chunks::<SZ>()
                    .map(u32::from_le_bytes)
                    .map(BlockAddress::new)
                    .nth(0) // FIXME: this is obviously incorrect - if the file is large enough, this always only reads the first double indirect block
                    .unwrap(),
                BlockPointerType::TripleIndirect => {
                    todo!("triple indirect pointers")
                }
            };

            // FIXME: if block_pointer is zero, we can only read a block of zeros ONLY IF THE FILE SYSTEM IS SPARSE

            let start_index = i * block_size;
            let end_index = start_index + block_size;
            let slice = &mut data[start_index..end_index];
            if let Some(block_pointer) = block_pointer {
                self.inner
                    .read()
                    .fs
                    .read_block(block_pointer, slice)
                    .map_err(|_| ReadError::IoError(IoError::HardwareError))?;
            } else {
                slice.fill(0);
            }
        }
        buffer.copy_from_slice(&data[relative_offset..relative_offset + buffer.len()]);

        Ok(buffer.len())
    }

    fn write_at(&mut self, _offset: u64, _buf: &dyn AsRef<[u8]>) -> Result<usize, WriteError> {
        todo!()
    }
}

enum BlockPointerType {
    Direct,
    SingleIndirect,
    DoubleIndirect,
    TripleIndirect,
}
