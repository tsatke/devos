use alloc::collections::btree_map::Entry::{Occupied, Vacant};
use alloc::collections::{BTreeMap, VecDeque};
use alloc::vec;
use alloc::vec::Vec;

use filesystem::BlockDevice;
use spin::RwLock;

pub struct CachingBlockDevice<T> {
    inner: RwLock<Inner<T>>,
}

struct CachedSector {
    dirty: bool,
    data: Vec<u8>,
}

struct Inner<T> {
    max_sector_count: usize,
    device: T,
    cached_sectors: BTreeMap<usize, CachedSector>,
    accessed_sectors: VecDeque<usize>,
}

impl<T> CachingBlockDevice<T> {
    pub fn new(device: T, max_sector_count: usize) -> Self {
        CachingBlockDevice {
            inner: RwLock::new(Inner {
                max_sector_count,
                device,
                cached_sectors: Default::default(),
                accessed_sectors: Default::default(),
            }),
        }
    }
}

impl<T> BlockDevice for CachingBlockDevice<T>
where
    T: BlockDevice,
{
    type Error = T::Error;

    fn sector_size(&self) -> usize {
        self.inner.read().device.sector_size()
    }

    fn sector_count(&self) -> usize {
        self.inner.read().device.sector_count()
    }

    fn read_sector(&self, sector_index: usize, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let mut inner = self.inner.write();
        inner.read_sector(sector_index, buf)
    }

    fn write_sector(&mut self, sector_index: usize, buf: &[u8]) -> Result<usize, Self::Error> {
        let mut inner = self.inner.write();
        inner.write_sector(sector_index, buf)
    }
}

impl<T> Inner<T>
where
    T: BlockDevice,
{
    fn read_sector(
        &mut self,
        sector_index: usize,
        buf: &mut [u8],
    ) -> Result<usize, <CachingBlockDevice<T> as BlockDevice>::Error> {
        if let Vacant(e) = self.cached_sectors.entry(sector_index) {
            // we don't have the sector in cache, so we need to load it from the device
            let mut sector = CachedSector {
                dirty: false,
                data: vec![0; self.device.sector_size()],
            };
            // read it  from the device
            let n_read = self.device.read_sector(sector_index, &mut sector.data)?;
            debug_assert_eq!(n_read, sector.data.len());

            buf.copy_from_slice(&sector.data);

            e.insert(sector);
            self.accessed_sectors.push_front(sector_index);

            self.evict_if_necessary()?;
        } else {
            // we already have the sector in cache, now move it to the front of the accessed list
            self.accessed_sectors.retain(|&x| x != sector_index);
            self.accessed_sectors.push_front(sector_index);
            // then read it into the read buffer
            let sector = self.cached_sectors.get(&sector_index).unwrap();
            buf.copy_from_slice(&sector.data);
        }

        Ok(buf.len())
    }

    fn write_sector(
        &mut self,
        sector_index: usize,
        buf: &[u8],
    ) -> Result<usize, <CachingBlockDevice<T> as BlockDevice>::Error> {
        match self.cached_sectors.entry(sector_index) {
            Vacant(e) => {
                // The sector is not in the cache, so we create a new one. We don't need to
                // read it from the device, because we are going to overwrite it anyway.
                let sector = CachedSector {
                    dirty: true,
                    data: buf.to_vec(),
                };
                e.insert(sector);
            }
            Occupied(mut e) => {
                // The sector is already in the cache, so we update it
                let sector = e.get_mut();
                sector.data.copy_from_slice(buf);
                sector.dirty = true;
            }
        }

        // Move the sector to the front of the accessed list
        self.accessed_sectors.retain(|&x| x != sector_index);
        self.accessed_sectors.push_front(sector_index);

        self.evict_if_necessary()?;

        Ok(buf.len())
    }

    fn evict_if_necessary(&mut self) -> Result<(), <CachingBlockDevice<T> as BlockDevice>::Error> {
        while self.accessed_sectors.len() > self.max_sector_count {
            let to_remove = self
                .accessed_sectors
                .pop_back()
                .expect("accessed_sectors is empty");
            let sector = self.cached_sectors.remove(&to_remove).unwrap();
            if sector.dirty {
                // If the removed sector is dirty, write it back to the device
                self.device.write_sector(to_remove, &sector.data)?;
            }
        }
        Ok(())
    }
}
