use core::ptr;

use x86_64::registers::control::{Cr3, Cr3Flags};
use x86_64::structures::paging::{
    Mapper, Page, PageSize, PageTable, PageTableFlags, PhysFrame, RecursivePageTable, Size4KiB,
    Translate,
};
use x86_64::structures::paging::mapper::{
    InvalidPageTable, MapperFlush, MapToError, TranslateResult, UnmapError,
};
use x86_64::VirtAddr;

use crate::{KERNEL_CODE_ADDR, process};
use crate::mem::physical::{FrameAllocatorDelegate, PhysicalMemoryManager};
use crate::process::vmm;

#[derive(Debug, Eq, PartialEq)]
pub struct AddressSpace {
    /// The virtual address of the level 4 page table **in this address space**.
    /// You can not dereference this and have the page table while this address
    /// space is not active. You can check whether it's active with [`AddressSpace::is_active`].
    level4_table_virtual_addr: VirtAddr,
    /// The physical address of the level 4 page table.
    level4_table_physical_frame: PhysFrame,
    /// The flags that are set in the Cr3 register. This may not be up to date, use
    /// [`AddressSpace::cr3flags`] to have a guaranteed up to date value while this address
    /// space is active.
    cr3flags: Cr3Flags,
}

// @dev AddressSpace must not be copy or clone, because it is essentially a pointer into
// memory, so copying or cloning an address space is aliasing.
impl ! Clone for AddressSpace {}

impl AddressSpace {
    pub fn allocate_new() -> Self {
        // allocate a new physical frame - this is valid for the entire machine
        let pt_frame = PhysicalMemoryManager::lock().allocate_frame().unwrap();
        // reserve a page in the current process, as we need to create a new page table,
        // but we'll free the interval later, after we're done with the page table setup
        let pt_interval = vmm()
            .reserve(Size4KiB::SIZE as usize)
            .unwrap();
        let pt_vaddr = pt_interval.start();
        let pt_page = Page::containing_address(pt_vaddr);

        // we map the page in the current process, so that we can modify the page table
        // for the new address space
        let current_process = process::current();
        let mut current_addr_space = current_process.address_space().write();
        unsafe {
            current_addr_space.map_to(
                pt_page,
                pt_frame,
                PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
            )
        }
            .unwrap()
            .flush();

        // create new page table
        let mut pt = PageTable::new();

        {
            // copy the kernel page table entries
            let kernel_start_index: u16 =
                Page::<Size4KiB>::containing_address(*KERNEL_CODE_ADDR.get().unwrap())
                    .p4_index()
                    .into();
            let kernel_range = kernel_start_index..512;

            let mut current_rec_pt = current_addr_space.get_recursive_page_table();
            let current_pt = &*current_rec_pt.level_4_table();
            for i in kernel_range {
                let pte = &current_pt[i as usize];
                pt[i as usize].set_addr(pte.addr(), pte.flags());
            }
        }

        let (pte_index, pte) = pt
            .iter_mut()
            .enumerate()
            .filter(|(_, e)| e.is_unused())
            .last()
            .unwrap();
        assert_ne!(pte_index, 0, "no free entries in new page table"); // if the last unused index is 0, we have a problem, since that is the entry that we need to keep free for userspace
        pte.set_frame(pt_frame, PageTableFlags::PRESENT | PageTableFlags::WRITABLE);

        // write the page table into the reserved page
        unsafe {
            // Safety: we just reserved the address and mapped it by hand, so this is safe to write to
            ptr::write(pt_vaddr.as_mut_ptr(), pt);
        }

        current_addr_space.unmap(pt_page).unwrap().1.flush(); // the physical frame that's "leaking" here is the frame containing the new page table
        drop(pt_interval); // keep the interval valid until after we've unmapped the page

        AddressSpace::new(
            pt_frame,
            Cr3Flags::empty(),
            TryInto::<u16>::try_into(pte_index).expect("page table index too large"),
        )
    }

    pub fn new(
        level4_table_physical_address: PhysFrame,
        cr3flags: Cr3Flags,
        recursive_index: u16,
    ) -> Self {
        let level4_table_virtual_addr = recursive_index_to_virtual_address(recursive_index);

        // we can't make sure that the virtual address is actually valid because we can't dereference
        // the value, we need to map it first
        // FIXME: validate the page table, this is an invariant of this type

        Self {
            level4_table_virtual_addr,
            level4_table_physical_frame: level4_table_physical_address,
            cr3flags,
        }
    }

    /// Determines whether this address space is currently active, i.e.
    /// whether the physical address of the level 4 page table of this
    /// address space is the current value of the [`Cr3`] register.
    pub fn is_active(&self) -> bool {
        Cr3::read().0 == self.level4_table_physical_frame
    }

    pub fn cr3_value(&self) -> usize {
        self.level4_table_physical_frame.start_address().as_u64() as usize
            | self.cr3flags().bits() as usize
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

    /// # Safety
    /// Mapping a page is inherently unsafe. See [`Mapper::map_to`] for more details.
    pub unsafe fn map_to(
        &mut self,
        page: Page,
        frame: PhysFrame,
        flags: PageTableFlags,
    ) -> Result<MapperFlush<Size4KiB>, MapToError<Size4KiB>> {
        let mut rpt = self.get_recursive_page_table();
        unsafe { rpt.map_to(page, frame, flags, &mut FrameAllocatorDelegate) }
    }

    pub fn unmap(
        &mut self,
        page: Page,
    ) -> Result<(PhysFrame<Size4KiB>, MapperFlush<Size4KiB>), UnmapError> {
        self.get_recursive_page_table().unmap(page)
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

fn recursive_index_to_virtual_address(recursive_index: u16) -> VirtAddr {
    let i = recursive_index as u64;
    VirtAddr::new(i << 39 | i << 30 | i << 21 | i << 12)
}
