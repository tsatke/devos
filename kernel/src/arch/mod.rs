use log::debug;

pub mod gdt;
pub mod idt;

pub fn init_no_heap() {
    gdt::init();

    debug!("arch initialized (pre-heap)");
}

pub fn init_with_heap() {
    idt::init();

    debug!("arch initialized (post-heap)");
}
