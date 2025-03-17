use log::debug;

pub mod gdt;
pub mod idt;

pub fn init_global() {
    gdt::init();

    debug!("arch initialized (global)");
}

pub fn init_cpu() {
    idt::init();

    debug!("arch initialized (per-cpu)");
}
