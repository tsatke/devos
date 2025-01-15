use crate::mem::physical::init_stage1;
use limine::request::MemoryMapRequest;
use log::debug;

#[used]
#[unsafe(link_section = ".requests")]
static MEMORY_MAP_REQUEST: MemoryMapRequest = MemoryMapRequest::new();

mod physical;
mod physical_stage1;

pub fn init() {
    let response = MEMORY_MAP_REQUEST
        .get_response()
        .expect("should have a memory map response");

    init_stage1(response.entries());

    /*
    1. create a recursive page table
    2. activate it
    3. map the kernel heap
    4. init the heap
    5. init stage 2
     */

    debug!("memory initialized")
}
