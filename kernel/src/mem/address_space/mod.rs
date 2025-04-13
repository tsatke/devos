use crate::limine::{HHDM_REQUEST, KERNEL_ADDRESS_REQUEST, MEMORY_MAP_REQUEST};
use crate::mem::phys::PhysicalMemory;
use crate::U64Ext;
use conquer_once::spin::OnceCell;
use core::fmt::{Debug, Formatter};
use limine::memory_map::EntryType;
use log::{debug, info};
use mapper::AddressSpaceMapper;
use spin::RwLock;
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::mapper::{
    MapToError, MappedFrame, MapperAllSizes, PageTableFrameMapping, TranslateResult,
};
use x86_64::structures::paging::page::PageRangeInclusive;
use x86_64::structures::paging::{
    MappedPageTable, Mapper, OffsetPageTable, Page, PageSize, PageTable, PageTableFlags, PhysFrame,
    RecursivePageTable, Translate,
};
use x86_64::{PhysAddr, VirtAddr};

mod mapper;

static KERNEL_ADDRESS_SPACE: OnceCell<AddressSpace> = OnceCell::uninit();
pub static RECURSIVE_INDEX: OnceCell<usize> = OnceCell::uninit();

pub fn init() {
    let (pt_vaddr, pt_frame) = make_mapping_recursive();
    let address_space = unsafe { AddressSpace::create_from(pt_frame, pt_vaddr) };
    KERNEL_ADDRESS_SPACE.init_once(|| address_space);
}

fn make_mapping_recursive() -> (VirtAddr, PhysFrame) {
    let hhdm_offset = HHDM_REQUEST
        .get_response()
        .expect("should have a HHDM response")
        .offset();

    let (level_4_table, level_4_table_frame) = {
        let frame = PhysicalMemory::allocate_frame().unwrap();
        let pt = unsafe {
            &mut *VirtAddr::new(frame.start_address().as_u64() + hhdm_offset)
                .as_mut_ptr::<PageTable>()
        };
        pt.zero();
        (pt, frame)
    };

    let mut current_pt = unsafe {
        OffsetPageTable::new(
            &mut *VirtAddr::new(Cr3::read().0.start_address().as_u64() + hhdm_offset)
                .as_mut_ptr::<PageTable>(),
            VirtAddr::new(hhdm_offset),
        )
    };

    let mut new_pt = {
        struct Offset(u64);
        unsafe impl PageTableFrameMapping for Offset {
            fn frame_to_pointer(&self, frame: PhysFrame) -> *mut PageTable {
                VirtAddr::new(frame.start_address().as_u64() + self.0).as_mut_ptr::<PageTable>()
            }
        }
        unsafe { MappedPageTable::new(level_4_table, Offset(hhdm_offset)) }
    };

    let kernel_addr = KERNEL_ADDRESS_REQUEST
        .get_response()
        .unwrap()
        .virtual_base();
    assert_eq!(
        1,
        MEMORY_MAP_REQUEST
            .get_response()
            .unwrap()
            .entries()
            .iter()
            .filter(|e| e.entry_type == EntryType::EXECUTABLE_AND_MODULES)
            .count()
    );
    let kernel_size = MEMORY_MAP_REQUEST
        .get_response()
        .unwrap()
        .entries()
        .iter()
        .find(|e| e.entry_type == EntryType::EXECUTABLE_AND_MODULES)
        .unwrap()
        .length;

    info!("remapping kernel");
    remap(
        &mut current_pt,
        &mut new_pt,
        VirtAddr::new(kernel_addr),
        kernel_size.into_usize(),
    );

    MEMORY_MAP_REQUEST
        .get_response()
        .unwrap()
        .entries()
        .iter()
        .filter(|e| e.entry_type == EntryType::BOOTLOADER_RECLAIMABLE)
        .for_each(|e| {
            remap(
                &mut current_pt,
                &mut new_pt,
                VirtAddr::new(e.base + hhdm_offset),
                e.length.into_usize(),
            );
        });

    let recursive_index = (0..512)
        .rposition(|p| level_4_table[p].is_unused())
        .expect("should have an unused index in the level 4 table");
    level_4_table[recursive_index].set_frame(
        level_4_table_frame,
        PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_EXECUTE,
    );
    let vaddr = recursive_index_to_virtual_address(recursive_index);
    debug!("recursive index: {recursive_index:?}, vaddr: {vaddr:p}");
    RECURSIVE_INDEX.init_once(|| recursive_index);

    level_4_table
        .iter_mut()
        .skip(256)
        .filter(|e| e.is_unused())
        .for_each(|e| {
            e.set_frame(
                PhysicalMemory::allocate_frame().unwrap(),
                PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_EXECUTE,
            );
        });

    info!("switching to recursive mapping");
    unsafe {
        let cr3_flags = Cr3::read().1;
        Cr3::write(level_4_table_frame, cr3_flags);
    }

    info!("done");

    (vaddr, level_4_table_frame)
}

fn remap(
    current_pt: &mut OffsetPageTable,
    new_pt: &mut impl MapperAllSizes,
    start_vaddr: VirtAddr,
    len: usize,
) {
    let mut current_addr = start_vaddr;

    while current_addr.as_u64() < start_vaddr.as_u64() + len as u64 {
        let result = current_pt.translate(current_addr);
        let TranslateResult::Mapped {
            frame,
            offset: _,
            flags,
        } = result
        else {
            unreachable!()
        };

        let flags = flags.intersection(
            PageTableFlags::PRESENT
                | PageTableFlags::WRITABLE
                | PageTableFlags::NO_EXECUTE
                | PageTableFlags::HUGE_PAGE,
        );

        let step = frame.size();
        unsafe {
            match frame {
                MappedFrame::Size4KiB(f) => {
                    let _ = new_pt
                        .map_to(
                            Page::containing_address(current_addr),
                            f,
                            flags,
                            &mut PhysicalMemory,
                        )
                        .unwrap();
                }
                MappedFrame::Size2MiB(f) => {
                    let _ = new_pt
                        .map_to(
                            Page::containing_address(current_addr),
                            f,
                            flags,
                            &mut PhysicalMemory,
                        )
                        .unwrap();
                }
                MappedFrame::Size1GiB(f) => {
                    let _ = new_pt
                        .map_to(
                            Page::containing_address(current_addr),
                            f,
                            flags,
                            &mut PhysicalMemory,
                        )
                        .unwrap();
                }
            }
        }
        current_addr += step;
    }
}

#[must_use]
pub const fn recursive_index_to_virtual_address(recursive_index: usize) -> VirtAddr {
    let i = recursive_index as u64;
    let addr = (i << 39) | (i << 30) | (i << 21) | (i << 12);

    let addr = sign_extend_vaddr(addr);

    VirtAddr::new(addr)
}

#[must_use]
pub const fn virt_addr_from_page_table_indices(indices: [u16; 4], offset: u64) -> VirtAddr {
    let addr = ((indices[0] as u64) << 39)
        | ((indices[1] as u64) << 30)
        | ((indices[2] as u64) << 21)
        | ((indices[3] as u64) << 12)
        | (offset & ((1 << 12) - 1));
    VirtAddr::new(sign_extend_vaddr(addr))
}

#[must_use]
pub const fn sign_extend_vaddr(vaddr: u64) -> u64 {
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_wrap)]
    let result = ((vaddr << 16) as i64 >> 16) as u64; // only works for 48-bit addresses
    result
}

pub struct AddressSpace {
    level4_frame: PhysFrame,
    inner: RwLock<AddressSpaceMapper>,
}

impl Debug for AddressSpace {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("AddressSpace")
            .field("level4_frame", &self.level4_frame)
            .field("active", &self.inner.read().is_active())
            .finish_non_exhaustive()
    }
}

impl AddressSpace {
    /// # Panics
    /// Panics if the kernel address space is not initialized yet.
    pub fn kernel() -> &'static Self {
        KERNEL_ADDRESS_SPACE
            .get()
            .expect("address space not initialized")
    }

    unsafe fn create_from(level4_frame: PhysFrame, level4_vaddr: VirtAddr) -> Self {
        Self {
            level4_frame,
            inner: RwLock::new(AddressSpaceMapper::new(level4_frame, level4_vaddr)),
        }
    }

    pub fn cr3_value(&self) -> usize {
        self.level4_frame.start_address().as_u64().into_usize()
    }

    #[allow(dead_code)]
    pub fn is_active(&self) -> bool {
        self.inner.read().is_active()
    }

    /// # Errors
    /// Returns an error if the page is already mapped or flags are invalid.
    #[allow(dead_code)]
    pub fn map<S: PageSize>(
        &self,
        page: Page<S>,
        frame: PhysFrame<S>,
        flags: PageTableFlags,
    ) -> Result<(), MapToError<S>>
    where
        for<'a> RecursivePageTable<'a>: Mapper<S>,
    {
        self.inner.write().map(page, frame, flags)
    }

    /// # Errors
    /// Returns an error if the pages are already mapped or flags are invalid.
    pub fn map_range<S: PageSize>(
        &self,
        pages: impl Into<PageRangeInclusive<S>>,
        frames: impl Iterator<Item = PhysFrame<S>>,
        flags: PageTableFlags,
    ) -> Result<(), MapToError<S>>
    where
        for<'a> RecursivePageTable<'a>: Mapper<S>,
    {
        self.inner.write().map_range(pages.into(), frames, flags)
    }

    pub fn unmap<S: PageSize>(&self, page: Page<S>) -> Option<PhysFrame<S>>
    where
        for<'a> RecursivePageTable<'a>: Mapper<S>,
    {
        self.inner.write().unmap(page)
    }

    pub fn unmap_range<S: PageSize>(
        &self,
        pages: impl Into<PageRangeInclusive<S>>,
        callback: impl Fn(PhysFrame<S>),
    ) where
        for<'a> RecursivePageTable<'a>: Mapper<S>,
    {
        self.inner.write().unmap_range(pages.into(), callback);
    }

    #[allow(dead_code)]
    pub fn translate(&self, vaddr: VirtAddr) -> Option<PhysAddr> {
        self.inner.read().translate(vaddr)
    }
}
