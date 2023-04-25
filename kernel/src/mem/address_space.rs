use x86_64::registers::control::{Cr3, Cr3Flags};
use x86_64::structures::paging::mapper::{
    InvalidPageTable, MapToError, MapperFlush, TranslateResult,
};
use x86_64::structures::paging::{
    Mapper, Page, PageTable, PageTableFlags, PhysFrame, RecursivePageTable, Size4KiB, Translate,
};
use x86_64::VirtAddr;

use crate::mem::FrameAllocatorDelegate;

pub struct AddressSpace {
    /// The virtual address of the level 4 page table **in this address space**.
    /// You can not dereference this and have the page table while this address
    /// space is not active. You can check whether it's active with [`AddressSpace::is_active`].
    level4_table_virtual_addr: VirtAddr,
    /// The physical address of the level 4 page table.
    level4_table_physical_address: PhysFrame,
    /// The flags that are set in the Cr3 register. This may not be up to date, use
    /// [`AddressSpace::cr3flags`] to have a guaranteed up to date value while this address
    /// space is active.
    cr3flags: Cr3Flags,
}

impl AddressSpace {
    pub fn new(
        level4_table_physical_address: PhysFrame,
        cr3flags: Cr3Flags,
        recursive_index: u16,
    ) -> Self {
        let level4_table_virtual_addr = {
            let i = recursive_index as u64;
            VirtAddr::new(i << 39 | i << 30 | i << 21 | i << 12)
        };

        // make sure that the virtual address is actually valid
        let _ = unsafe { as_recursive_page_table(level4_table_virtual_addr) }
            .expect("expect a valid recursive page table");

        Self {
            level4_table_virtual_addr,
            level4_table_physical_address,
            cr3flags,
        }
    }

    /// Determines whether this address space is currently active, i.e.
    /// whether the physical address of the level 4 page table of this
    /// address space is the current value of the [`Cr3`] register.
    pub fn is_active(&self) -> bool {
        Cr3::read().0 == self.level4_table_physical_address
    }

    /// Returns the current [`Cr3Flags`] if this address space is active,
    /// or the stored value of the cr3flags if this address space
    /// is currently not active.
    pub fn cr3flags(&self) -> Cr3Flags {
        if self.is_active() {
            Cr3::read().1
        } else {
            self.cr3flags
        }
    }

    pub unsafe fn map_to(
        &mut self,
        page: Page,
        frame: PhysFrame,
        flags: PageTableFlags,
    ) -> Result<MapperFlush<Size4KiB>, MapToError<Size4KiB>> {
        let mut rpt = self.get_recursive_page_table();
        unsafe { rpt.map_to(page, frame, flags, &mut FrameAllocatorDelegate) }
    }

    pub fn translate(&self, addr: VirtAddr) -> TranslateResult {
        self.get_recursive_page_table().translate(addr)
    }

    fn get_recursive_page_table(&self) -> RecursivePageTable<'static> {
        unsafe {
            // Safety: we checked that the address is valid when this address space was created
            as_recursive_page_table(self.level4_table_virtual_addr)
        }
        .expect("invalid page table vaddr")
    }
}

/// # Safety
/// The caller must ensure that the address is valid for reads
/// from `vaddr` to `vaddr + size_of::<PageTable>()`.
unsafe fn as_recursive_page_table(
    vaddr: VirtAddr,
) -> Result<RecursivePageTable<'static>, InvalidPageTable> {
    let pt: &mut PageTable = unsafe { &mut *(vaddr.as_mut_ptr()) };
    RecursivePageTable::new(pt)
}
