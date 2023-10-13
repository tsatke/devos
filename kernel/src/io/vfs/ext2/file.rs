use crate::io::vfs::ext2::InnerHandle;
use crate::io::vfs::{File, InodeBase, InodeNum, IoError, ReadError, Stat, WriteError};
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
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
        match block_index {
            0..12 => BlockPointerType::Direct,
            x if x >= 12 && x < hi_single_indirect => BlockPointerType::SingleIndirect,
            x if x >= hi_single_indirect && x < hi_double_indirect => {
                BlockPointerType::DoubleIndirect
            }
            x if x >= hi_double_indirect && x < hi_triple_indirect => {
                BlockPointerType::TripleIndirect
            }
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
        let guard = &mut self.inner.read();
        for i in 0..block_count {
            let read_block_index = (start_block + i as u32) as usize;
            let block_pointer = match self.determine_block_pointer_type(read_block_index) {
                BlockPointerType::Direct => self.file.direct_ptr(read_block_index).unwrap(),
                BlockPointerType::SingleIndirect => {
                    let pointer_block = self.file.single_indirect_ptr().unwrap();
                    let mut pointer_data = [0_u8; 4];
                    guard
                        .fs
                        .read_block(pointer_block, &mut pointer_data)
                        .map_err(|_| ReadError::IoError(IoError::HardwareError))?;
                    BlockAddress::new(u32::from_le_bytes(pointer_data)).unwrap()
                }
                BlockPointerType::DoubleIndirect => {
                    todo!("double indirect pointers")
                }
                BlockPointerType::TripleIndirect => {
                    todo!("triple indirect pointers")
                }
            };

            let start_index = i * block_size;
            let end_index = start_index + block_size;
            guard
                .fs
                .read_block(block_pointer, &mut data[start_index..end_index])
                .map_err(|_| ReadError::IoError(IoError::HardwareError))?;
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
