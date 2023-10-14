use crate::io::vfs::ext2::InnerHandle;
use crate::io::vfs::{File, InodeBase, InodeNum, IoError, ReadError, Stat, WriteError};
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
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

    fn read_blocks(
        &self,
        start_block: usize,
        end_block: usize,
        buf: &mut [u8],
    ) -> Result<(), ReadError> {
        let block_size = self.inner.read().fs.superblock().block_size() as usize;
        let pointers_per_block = block_size / 4;

        let direct_end = 11; // 12 direct pointers
        let single_start = direct_end + 1;
        let single_end = single_start + pointers_per_block;
        let single_count = single_end - single_start;
        let double_start = single_end;
        let double_end = double_start + pointers_per_block.pow(2);
        let double_count = double_end - double_start;
        let triple_start = double_end;

        let mut pointer_block_cache = BTreeMap::<BlockAddress, Vec<u8>>::new(); // FIXME: this can grow quite large e.g. a lot of triple indirect pointers are used

        for (i, block) in (start_block..=end_block).enumerate() {
            let block_data = &mut buf[i * block_size..(i + 1) * block_size];
            let block_pointer = if block < 12 {
                self.file.direct_ptr(block)
            } else {
                // The block index will be converted to a "path" like 3/2/5, which would
                // mean the third triple indirect entry, the second double indirect entry,
                // and the fifth single indirect entry.
                // Or 2/1, which would mean the second double indirect entry, and the first
                // single indirect entry.
                let mut indirect_path = Vec::<usize>::with_capacity(3);
                if block >= triple_start {
                    let triple_offset = block - triple_start;
                    let triple_index = triple_offset / double_count;
                    let double_index = (triple_offset % double_count) / single_count;
                    let single_index = (triple_offset % double_count) % single_count;
                    indirect_path.push(triple_index);
                    indirect_path.push(double_index);
                    indirect_path.push(single_index);
                } else if block >= double_start {
                    let double_offset = block - double_start;
                    let double_index = double_offset / single_count;
                    let single_index = double_offset % single_count;
                    indirect_path.push(double_index);
                    indirect_path.push(single_index);
                } else if block >= single_start {
                    let single_offset = block - single_start;
                    indirect_path.push(single_offset);
                }

                // this is the starting point for walking the path
                let mut block_pointer = match indirect_path.len() {
                    1 => self.file.single_indirect_ptr(),
                    2 => self.file.double_indirect_ptr(),
                    3 => self.file.triple_indirect_ptr(),
                    _ => panic!("invalid indirect path"),
                }
                .expect("indirect pointer block not allocated"); // FIXME: this can probably happen for sparse file systems
                for indirect_path_segment in indirect_path {
                    let pointer_data =
                        pointer_block_cache.entry(block_pointer).or_insert_with(|| {
                            let mut data = vec![0_u8; block_size];
                            self.inner
                                .read()
                                .fs
                                .read_block(block_pointer, &mut data)
                                .expect("failed to read single indirect pointer block");
                            data
                        });
                    const SZ: usize = size_of::<BlockAddress>();
                    block_pointer = pointer_data
                        .iter()
                        .copied()
                        .array_chunks::<SZ>()
                        .map(u32::from_le_bytes)
                        .map(BlockAddress::new)
                        .nth(indirect_path_segment)
                        .unwrap()
                        .expect("failed to get indirect pointer") // FIXME: this can probably happen for sparse file systems
                }
                Some(block_pointer)
            };

            if let Some(block_pointer) = block_pointer {
                self.inner
                    .read()
                    .fs
                    .read_block(block_pointer, block_data)
                    .map_err(|_| ReadError::IoError(IoError::HardwareError))?;
            } else {
                block_data.fill(0);
            }
        }

        Ok(())
    }
}

impl<T> InodeBase for Ext2File<T>
where
    T: BlockDevice + Send + Sync,
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
        let mut data: Vec<u8> = vec![0_u8; block_count * block_size]; // TODO: avoid allocation - maybe try to only allocate the first and last block if the read is not aligned, but read the rest directly into the buffer
        self.read_blocks(start_block as usize, end_block as usize, &mut data)?;
        buffer.copy_from_slice(&data[relative_offset..relative_offset + buffer.len()]);

        Ok(buffer.len())
    }

    fn write_at(&mut self, _offset: u64, _buf: &dyn AsRef<[u8]>) -> Result<usize, WriteError> {
        todo!()
    }
}
