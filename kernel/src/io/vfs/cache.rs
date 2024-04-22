use alloc::collections::{BTreeMap, VecDeque};
use alloc::collections::btree_map::Entry::Vacant;
use alloc::vec;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicUsize, Ordering};

use filesystem::BlockDevice;
use spin::RwLock;

pub struct CachingBlockDevice<T> {
    device: T,
    max_sector_count: usize,
    inner: RwLock<Inner>,

    cache_hits: AtomicUsize,
    cache_misses: AtomicUsize,
}

struct CachedSector {
    dirty: bool,
    data: Vec<u8>,
}

struct Inner {
    cached_sectors: BTreeMap<usize, CachedSector>,
    accessed_sectors: VecDeque<usize>,
}

impl<T> CachingBlockDevice<T>
{
    pub fn new(device: T, max_sector_count: usize) -> Self {
        CachingBlockDevice {
            device,
            max_sector_count,
            inner: RwLock::new(
                Inner {
                    cached_sectors: Default::default(),
                    accessed_sectors: Default::default(),
                }
            ),
            cache_hits: AtomicUsize::new(0),
            cache_misses: AtomicUsize::new(0),
        }
    }

    /// Returns the number of cache hits and misses as tuple `(hits, misses)`.
    pub fn cache_stats(&self) -> (usize, usize) {
        (self.cache_hits.load(Ordering::Relaxed), self.cache_misses.load(Ordering::Relaxed))
    }
}

impl<T> BlockDevice for CachingBlockDevice<T>
    where
        T: BlockDevice,
{
    type Error = T::Error;

    fn sector_size(&self) -> usize {
        self.device.sector_size()
    }

    fn sector_count(&self) -> usize {
        self.device.sector_count()
    }

    fn read_sector(&self, sector_index: usize, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let mut inner = self.inner.write();

        if let Vacant(e) = inner.cached_sectors.entry(sector_index) {
            self.cache_misses.fetch_add(1, Ordering::Relaxed);

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
            inner.accessed_sectors.push_front(sector_index);

            while inner.accessed_sectors.len() > self.max_sector_count {
                let to_remove = inner.accessed_sectors.pop_back().expect("accessed_sectors is empty");
                let sector = inner.cached_sectors.remove(&to_remove).unwrap();
                if sector.dirty {
                    todo!("write back to device");
                }
            }
        } else {
            self.cache_hits.fetch_add(1, Ordering::Relaxed);

            // we already have the sector in cache, now move it to the front of the accessed list
            inner.accessed_sectors.retain(|&x| x != sector_index);
            inner.accessed_sectors.push_front(sector_index);
            // then read it into the read buffer
            let sector = inner.cached_sectors.get(&sector_index).unwrap();
            buf.copy_from_slice(&sector.data);
        }

        Ok(buf.len())
    }

    fn write_sector(&mut self, _sector_index: usize, _buf: &[u8]) -> Result<usize, Self::Error> {
        todo!()
    }
}