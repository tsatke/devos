use crate::mcore::mtask::process::Process;
use crate::mem::phys::PhysicalMemory;
use crate::mem::virt::OwnedSegment;
use crate::mem::virt::VirtualMemoryAllocator;
use crate::{U64Ext, UsizeExt};
use alloc::sync::Arc;
use core::alloc::Layout;
use core::fmt::{Debug, Formatter};
use core::marker::PhantomData;
use core::ops::Deref;
use core::slice::{from_raw_parts, from_raw_parts_mut};
use kernel_memapi::{Allocation, Location, MemoryApi, UserAccessible, WritableAllocation};
use virtual_memory_manager::Segment;
use x86_64::VirtAddr;
use x86_64::structures::paging::{PageSize, PageTableFlags, Size4KiB};

#[derive(Clone)]
pub struct LowerHalfMemoryApi {
    process: Arc<Process>,
}

impl LowerHalfMemoryApi {
    pub fn new(process: Arc<Process>) -> Self {
        Self { process }
    }
}

impl MemoryApi for LowerHalfMemoryApi {
    type ReadonlyAllocation = LowerHalfAllocation<Readonly>;
    type WritableAllocation = LowerHalfAllocation<Writable>;
    type ExecutableAllocation = LowerHalfAllocation<Executable>;

    fn allocate(
        &mut self,
        location: Location,
        layout: Layout,
        user_accessible: UserAccessible,
    ) -> Option<Self::WritableAllocation> {
        assert!(layout.align() <= Size4KiB::SIZE.into_usize());

        let (start, segment) = match location {
            Location::Anywhere => {
                let segment = self
                    .process
                    .vmm()
                    .reserve(layout.size().div_ceil(Size4KiB::SIZE.into_usize()))?;
                (segment.start, segment)
            }
            Location::Fixed(v) => {
                let aligned_start_addr = v.align_down(Size4KiB::SIZE);
                let aligned_end_addr = (v + layout.size().into_u64()).align_up(Size4KiB::SIZE);
                let segment =
                    Segment::new(aligned_start_addr, aligned_end_addr - aligned_start_addr);
                let vmm = self.process.vmm();
                let segment = vmm.mark_as_reserved(segment).ok()?;
                (v, segment)
            }
        };

        self.process
            .address_space()
            .map_range::<Size4KiB>(
                &*segment,
                PhysicalMemory::allocate_frames_non_contiguous(),
                PageTableFlags::PRESENT
                    | PageTableFlags::WRITABLE
                    | PageTableFlags::NO_EXECUTE
                    | if user_accessible == UserAccessible::Yes {
                        PageTableFlags::USER_ACCESSIBLE
                    } else {
                        PageTableFlags::empty()
                    },
            )
            .ok()?;

        Some(LowerHalfAllocation {
            start,
            layout,
            inner: Inner {
                process: self.process.clone(),
                segment,
            },
            _typ: PhantomData,
        })
    }

    fn make_executable(
        &mut self,
        allocation: Self::WritableAllocation,
    ) -> Result<Self::ExecutableAllocation, Self::WritableAllocation> {
        let res = self.process.address_space().remap_range::<Size4KiB, _>(
            &*allocation.segment,
            |mut flags| {
                flags.remove(PageTableFlags::WRITABLE);
                flags.remove(PageTableFlags::NO_EXECUTE);
                flags
            },
        );
        if res.is_err() {
            return Err(allocation);
        }

        Ok(LowerHalfAllocation {
            start: allocation.start,
            layout: allocation.layout,
            inner: allocation.inner,
            _typ: PhantomData,
        })
    }

    fn make_writable(
        &mut self,
        allocation: Self::ExecutableAllocation,
    ) -> Result<Self::WritableAllocation, Self::ExecutableAllocation> {
        let res = self.process.address_space().remap_range::<Size4KiB, _>(
            &*allocation.segment,
            |mut flags| {
                flags.insert(PageTableFlags::WRITABLE);
                flags.insert(PageTableFlags::NO_EXECUTE);
                flags
            },
        );
        if res.is_err() {
            return Err(allocation);
        }

        Ok(LowerHalfAllocation {
            start: allocation.start,
            layout: allocation.layout,
            inner: allocation.inner,
            _typ: PhantomData,
        })
    }

    fn make_readonly(
        &mut self,
        allocation: Self::WritableAllocation,
    ) -> Result<Self::ReadonlyAllocation, Self::WritableAllocation> {
        let res = self.process.address_space().remap_range::<Size4KiB, _>(
            &*allocation.segment,
            |mut flags| {
                flags.remove(PageTableFlags::WRITABLE);
                flags.insert(PageTableFlags::NO_EXECUTE);
                flags
            },
        );
        if res.is_err() {
            return Err(allocation);
        }

        Ok(LowerHalfAllocation {
            start: allocation.start,
            layout: allocation.layout,
            inner: allocation.inner,
            _typ: PhantomData,
        })
    }
}

trait Sealed {}
#[allow(private_bounds)]
pub trait AllocationType: Sealed {}
#[derive(Debug)]
pub struct Readonly;
impl Sealed for Readonly {}
impl AllocationType for Readonly {}
#[derive(Debug)]
pub struct Writable;
impl Sealed for Writable {}
impl AllocationType for Writable {}
#[derive(Debug)]
pub struct Executable;
impl Sealed for Executable {}
impl AllocationType for Executable {}

pub struct LowerHalfAllocation<T> {
    start: VirtAddr,
    layout: Layout,
    inner: Inner,
    _typ: PhantomData<T>,
}

impl<T: AllocationType> LowerHalfAllocation<T> {
    #[must_use]
    pub fn start(&self) -> VirtAddr {
        self.start
    }

    #[allow(clippy::len_without_is_empty)]
    #[must_use]
    pub fn len(&self) -> usize {
        self.layout.size()
    }
}

pub struct Inner {
    segment: OwnedSegment<'static>,
    process: Arc<Process>,
}

impl<T: AllocationType> Deref for LowerHalfAllocation<T> {
    type Target = Inner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: AllocationType> Debug for LowerHalfAllocation<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("LowerHalfAllocation")
            .field("process_id", &self.process.pid())
            .field("segment", &self.segment)
            .field("typ", &self._typ)
            .finish_non_exhaustive()
    }
}

impl<T: AllocationType> AsRef<[u8]> for LowerHalfAllocation<T> {
    fn as_ref(&self) -> &[u8] {
        let ptr = self.start.as_ptr();
        unsafe { from_raw_parts(ptr, self.layout.size()) }
    }
}

impl<T: AllocationType> Allocation for LowerHalfAllocation<T> {
    fn layout(&self) -> Layout {
        self.layout
    }
}

impl AsMut<[u8]> for LowerHalfAllocation<Writable> {
    fn as_mut(&mut self) -> &mut [u8] {
        let ptr = self.start.as_mut_ptr();
        unsafe { from_raw_parts_mut(ptr, self.layout.size()) }
    }
}

impl WritableAllocation for LowerHalfAllocation<Writable> {}

impl Drop for Inner {
    fn drop(&mut self) {
        self.process
            .address_space()
            .unmap_range::<Size4KiB>(&*self.segment, PhysicalMemory::deallocate_frame);
    }
}
