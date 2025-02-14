use crate::limine::HHDM_REQUEST;
use conquer_once::spin::OnceCell;
use mapper::AddressSpaceMapper;
use spin::RwLock;
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::mapper::MapToError;
use x86_64::structures::paging::page::PageRangeInclusive;
use x86_64::structures::paging::{
    Mapper, Page, PageSize, PageTable, PageTableFlags, PhysFrame, RecursivePageTable,
};
use x86_64::{PhysAddr, VirtAddr};

mod mapper;

static KERNEL_ADDRESS_SPACE: OnceCell<AddressSpace> = OnceCell::uninit();

pub fn init() {
    let (pt_vaddr, pt_frame) = make_mapping_recursive();
    let address_space = unsafe { AddressSpace::create_from(pt_frame, pt_vaddr) };
    KERNEL_ADDRESS_SPACE.init_once(|| address_space);
}

fn make_mapping_recursive() -> (VirtAddr, PhysFrame) {
    // switch to a recursive page table
    let offset = HHDM_REQUEST.get_response().unwrap().offset();
    let (cr3_frame, _) = Cr3::read();
    let cr3_phys_addr = cr3_frame.start_address();
    let cr3_virt_addr = VirtAddr::new(cr3_phys_addr.as_u64() + offset);
    let current_pt = unsafe { &mut *cr3_virt_addr.as_mut_ptr::<PageTable>() };
    let recursive_index = 510; // TODO: find a free index
    current_pt[recursive_index].set_frame(
        cr3_frame,
        PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
    );
    let pt_vaddr = recursive_index_to_virtual_address(recursive_index);

    (pt_vaddr, cr3_frame)
}

fn recursive_index_to_virtual_address(recursive_index: usize) -> VirtAddr {
    let i = recursive_index as u64;
    let addr = (i << 39) | (i << 30) | (i << 21) | (i << 12);

    let addr = sign_extend_vaddr(addr);

    VirtAddr::new(addr)
}

pub const fn virt_addr_from_page_table_indices(indices: [u16; 4], offset: u64) -> VirtAddr {
    let addr = ((indices[0] as u64) << 39)
        | ((indices[1] as u64) << 30)
        | ((indices[2] as u64) << 21)
        | ((indices[3] as u64) << 12)
        | (offset & ((1 << 12) - 1));
    VirtAddr::new(sign_extend_vaddr(addr))
}

pub const fn sign_extend_vaddr(vaddr: u64) -> u64 {
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_wrap)]
    let result = ((vaddr << 16) as i64 >> 16) as u64; // only works for 48-bit addresses
    result
}

#[derive(Debug)]
pub struct AddressSpace {
    inner: RwLock<AddressSpaceMapper>,
}

impl AddressSpace {
    pub fn kernel() -> &'static Self {
        KERNEL_ADDRESS_SPACE
            .get()
            .expect("address space not initialized")
    }

    unsafe fn create_from(level4_frame: PhysFrame, level4_vaddr: VirtAddr) -> Self {
        Self {
            inner: AddressSpaceMapper::new(level4_frame, level4_vaddr).into(),
        }
    }

    #[allow(dead_code)]
    pub fn is_active(&self) -> bool {
        self.inner.read().is_active()
    }

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

    #[allow(dead_code)]
    pub fn translate(&self, vaddr: VirtAddr) -> Option<PhysAddr> {
        self.inner.read().translate(vaddr)
    }
}
