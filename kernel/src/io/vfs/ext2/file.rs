use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use core::mem::size_of;

use derive_more::Constructor;
use ext2::{BlockAddress, InodeAddress};
use filesystem::BlockDevice;
use spin::RwLock;

use crate::io::vfs::error::{Result, VfsError};
use crate::io::vfs::Stat;

#[derive(Constructor)]
pub struct Ext2Inode<T> {
    fs: Arc<RwLock<ext2::Ext2Fs<T>>>,
    inode_num: InodeAddress,
    inner: ext2::Inode,
}

impl<T> Ext2Inode<T>
where
    T: BlockDevice,
{
    pub fn read(&self, buf: &mut [u8], offset: usize) -> Result<usize> {
        let block_size = self.fs.read().superblock().block_size();
        let offset = offset as u32;

        let start_block = offset / block_size;
        let end_block = (offset + buf.len() as u32 - 1) / block_size;
        let relative_offset = (offset % block_size) as usize;
        let block_count = (end_block - start_block + 1) as usize;

        // read blocks
        let mut data: Vec<u8> = vec![0_u8; block_count * block_size as usize]; // TODO: avoid allocation - maybe try to only allocate the first and last block if the read is not aligned, but read the rest directly into the buffer
        self.read_blocks(start_block as usize, end_block as usize, &mut data)?;
        buf.copy_from_slice(&data[relative_offset..relative_offset + buf.len()]);

        Ok(buf.len())
    }

    pub fn write(&mut self, _buf: &[u8], _offset: usize) -> Result<usize> {
        todo!()
    }

    pub fn stat(&self) -> Result<Stat> {
        Ok(Stat {
            inode: self.inode_num.get() as u64, // TODO: is this correct?
            size: self.inner.len() as u64,
            ..Default::default() // TODO: fill in the rest
        })
    }
}

impl<T> Ext2Inode<T>
where
    T: BlockDevice,
{
    fn read_blocks(&self, start_block: usize, end_block: usize, buf: &mut [u8]) -> Result<()> {
        let block_size = self.fs.read().superblock().block_size() as usize;
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
                self.inner.direct_ptr(block)
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
                    1 => self.inner.single_indirect_ptr(),
                    2 => self.inner.double_indirect_ptr(),
                    3 => self.inner.triple_indirect_ptr(),
                    _ => panic!("invalid indirect path"),
                }
                .expect("indirect pointer block not allocated"); // FIXME: this can probably happen for sparse file systems
                for indirect_path_segment in indirect_path {
                    let pointer_data =
                        pointer_block_cache.entry(block_pointer).or_insert_with(|| {
                            let mut data = vec![0_u8; block_size];
                            self.fs
                                .read()
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
                self.fs
                    .read()
                    .read_block(block_pointer, block_data)
                    .map_err(|_| VfsError::ReadError)?;
            } else {
                block_data.fill(0);
            }
        }

        Ok(())
    }
}
