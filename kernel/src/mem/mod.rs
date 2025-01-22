use crate::mem::address_space::AddressSpace;
use crate::mem::heap::Heap;
use limine::request::{HhdmRequest, MemoryMapRequest, PagingModeRequest};
use log::info;

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

    address_space::init();

    let address_space = AddressSpace::kernel();

    heap::init(address_space);

    phys::init_stage2();

    info!("memory initialized, {:?}", Heap);
}
