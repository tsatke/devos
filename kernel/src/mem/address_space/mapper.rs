use crate::mem::phys::PhysicalMemory;
use log::warn;
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::mapper::MapToError;
use x86_64::structures::paging::page::PageRangeInclusive;
use x86_64::structures::paging::{
    Mapper, Page, PageSize, PageTable, PageTableFlags, PhysFrame, RecursivePageTable,
};
use x86_64::VirtAddr;

#[derive(Debug)]
pub struct AddressSpaceMapper {
    level4_frame: PhysFrame,
    level4_vaddr: VirtAddr,
}

impl AddressSpaceMapper {
    pub fn new(level4_frame: PhysFrame, level4_vaddr: VirtAddr) -> Self {
        Self {
            level4_frame,
            level4_vaddr,
        }
    }

    pub fn is_active(&self) -> bool {
        self.level4_frame == Cr3::read().0
    }

    pub fn map<S: PageSize>(
        &mut self,
        page: Page<S>,
        frame: PhysFrame<S>,
        flags: PageTableFlags,
    ) -> Result<(), MapToError<S>>
    where
        for<'a> RecursivePageTable<'a>: Mapper<S>,
    {
        assert!(self.is_active()); // TODO: support mapping into non-active address spaces

        #[cfg(debug_assertions)]
        {
            if !flags.contains(PageTableFlags::PRESENT) {
                warn!(
                    "mapping {:p} to {:p} without PRESENT flag",
                    page.start_address(),
                    frame.start_address()
                );
            }
        }

        let mut page_table = self.recursive_page_table();
        unsafe {
            page_table
                .map_to(page, frame, flags, &mut PhysicalMemory)?
                .flush();
        }

        Ok(())
    }

    pub fn map_range<S: PageSize>(
        &mut self,
        pages: PageRangeInclusive<S>,
        frames: impl Iterator<Item = PhysFrame<S>>,
        flags: PageTableFlags,
    ) -> Result<(), MapToError<S>>
    where
        for<'a> RecursivePageTable<'a>: Mapper<S>,
    {
        assert!(self.is_active()); // TODO: support mapping into non-active address spaces

        let frames = frames.into_iter();
        for (page, frame) in pages.zip(frames) {
            self.map(page, frame, flags)?;
        }

        Ok(())
    }

    fn recursive_page_table(&mut self) -> RecursivePageTable {
        let pt = unsafe { &mut *self.level4_vaddr.as_mut_ptr::<PageTable>() };
        RecursivePageTable::new(pt).expect("should be a valid recursive page table")
    }
}
