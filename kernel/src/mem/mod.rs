use crate::mem::address_space::AddressSpace;
use crate::mem::heap::Heap;
use crate::mem::phys::PhysicalMemory;
use limine::request::{HhdmRequest, MemoryMapRequest, PagingModeRequest};
use log::info;
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::{
    Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, PhysFrame, Size4KiB,
};
use x86_64::VirtAddr;

mod address_space;
mod heap;
mod phys;

#[used]
#[unsafe(link_section = ".requests")]
static MEMORY_MAP_REQUEST: MemoryMapRequest = MemoryMapRequest::new();

#[used]
#[unsafe(link_section = ".requests")]
static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();

#[used]
#[unsafe(link_section = ".requests")]
static PAGING_MODE_REQUEST: PagingModeRequest = PagingModeRequest::new();

pub fn init() {
    let response = MEMORY_MAP_REQUEST
        .get_response()
        .expect("should have a memory map response");

    phys::init_stage1(response.entries());

    let (pt_vaddr, pt_frame) = remap_with_recursive_page_table();
    let address_space = unsafe { AddressSpace::create_from(pt_frame, pt_vaddr) };
    assert!(address_space.is_active());

    heap::init(address_space);

    phys::init_stage2();

    info!("memory initialized, {:?}", Heap);
}

fn remap_with_recursive_page_table() -> (VirtAddr, PhysFrame) {
    // switch to a recursive page table
    let offset = HHDM_REQUEST.get_response().unwrap().offset();
    let (cr3_frame, cr3_flags) = Cr3::read();
    let cr3_paddr = cr3_frame.start_address();
    let cr3_vaddr = VirtAddr::new(cr3_paddr.as_u64() + offset);
    let current_pt = unsafe { &mut *cr3_vaddr.as_mut_ptr::<PageTable>() };
    let mut offset_pt = unsafe { OffsetPageTable::new(current_pt, VirtAddr::new(offset)) };

    let pt_frame = PhysicalMemory::allocate_frame().unwrap();
    let recursive_index = 510;
    let pt_vaddr = recursive_index_to_virtual_address(recursive_index);

    unsafe {
        offset_pt
            .map_to(
                Page::<Size4KiB>::containing_address(pt_vaddr),
                pt_frame,
                PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
                &mut PhysicalMemory,
            )
            .unwrap()
            .flush();
    }

    let new_pt = unsafe { &mut *(pt_vaddr.as_mut_ptr::<PageTable>()) };
    new_pt.zero();
    new_pt[256] = current_pt[256].clone();
    new_pt[recursive_index].set_frame(pt_frame, PageTableFlags::PRESENT | PageTableFlags::WRITABLE);
    new_pt[511] = current_pt[511].clone();

    unsafe { Cr3::write(pt_frame, cr3_flags) };

    (pt_vaddr, pt_frame)
}

fn recursive_index_to_virtual_address(recursive_index: usize) -> VirtAddr {
    let i = recursive_index as u64;
    let addr = i << 39 | i << 30 | i << 21 | i << 12;

    let addr = ((addr << 16) as i64 >> 16) as u64; // correctly sign extend the address - 48-bit

    VirtAddr::new(addr)
}

pub const fn virt_addr_from_page_table_indices(indices: &[u16; 4], offset: u64) -> VirtAddr {
    let addr = (indices[0] as u64) << 39
        | (indices[1] as u64) << 30
        | (indices[2] as u64) << 21
        | (indices[3] as u64) << 12
        | (offset & ((1 << 12) - 1));
    VirtAddr::new(sign_extend_vaddr(addr))
}

pub const fn sign_extend_vaddr(vaddr: u64) -> u64 {
    ((vaddr << 16) as i64 >> 16) as u64 // only works for 48-bit addresses
}
