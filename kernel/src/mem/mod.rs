use crate::mem::phys::PhysicalMemory;
use core::iter::zip;
use limine::paging::Mode;
use limine::request::{
    HhdmRequest, KernelAddressRequest, KernelFileRequest, MemoryMapRequest, PagingModeRequest,
};
use log::{debug, info};
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::mapper::CleanUp;
use x86_64::structures::paging::{
    Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, PageTableIndex, PhysFrame,
    RecursivePageTable, Size4KiB, Translate,
};
use x86_64::{PhysAddr, VirtAddr};

mod heap;
mod phys;
mod virt;

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

    // switch to a recursive page table
    let offset = HHDM_REQUEST.get_response().unwrap().offset();
    let (cr3_frame, cr3_flags) = Cr3::read();
    let cr3_paddr = cr3_frame.start_address();
    info!("cr3: {:#x}", cr3_paddr.as_u64());
    let cr3_vaddr = VirtAddr::new(cr3_paddr.as_u64() + offset);
    info!("cr3_vaddr: {:#x}", cr3_vaddr.as_u64());
    let current_pt = unsafe { &mut *cr3_vaddr.as_mut_ptr::<PageTable>() };
    let mut offset_pt = unsafe { OffsetPageTable::new(current_pt, VirtAddr::new(offset)) };

    let pt_frame = PhysicalMemory::allocate_frame().unwrap();
    info!("allocated frame: {:p}", pt_frame.start_address());
    let recursive_index = 510;
    let pt_vaddr = recursive_index_to_virtual_address(recursive_index);
    let pt_paddr = pt_frame.start_address();

    info!("mapping {pt_vaddr:p} to {pt_paddr:p}");
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

    let mut new_pt = unsafe { &mut *(pt_vaddr.as_mut_ptr::<PageTable>()) };
    new_pt.zero();
    new_pt[256] = current_pt[256].clone();
    new_pt[511] = current_pt[511].clone();

    unsafe { Cr3::write(pt_frame, cr3_flags) };
    // TODO: remap the kernel with a recursive page table

    heap::init();

    phys::init_stage2();

    debug!("memory initialized")
}

fn recursive_index_to_virtual_address(recursive_index: usize) -> VirtAddr {
    let i = recursive_index as u64;
    let addr = i << 39 | i << 30 | i << 21 | i << 12;

    debug_assert!(Mode::FOUR_LEVEL == PAGING_MODE_REQUEST.mode());
    let addr = ((addr << 16) as i64 >> 16) as u64; // correctly sign extend the address - 48-bit

    VirtAddr::new(addr)
}
