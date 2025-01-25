use crate::limine::MEMORY_MAP_REQUEST;
use crate::mem::address_space::AddressSpace;
use crate::mem::heap::Heap;
use log::info;

pub mod address_space;
mod heap;
pub mod phys;
pub mod virt;

pub fn init() {
    let response = MEMORY_MAP_REQUEST
        .get_response()
        .expect("should have a memory map response");

    phys::init_stage1(response.entries());

    address_space::init();

    let address_space = AddressSpace::kernel();

    heap::init(address_space);

    virt::init();

    phys::init_stage2();

    info!("memory initialized, {:?}", Heap);
}
