use conquer_once::spin::Lazy;
use core::ptr::addr_of;
use x86_64::instructions::tables::load_tss;
use x86_64::registers::segmentation::{Segment, CS, DS};
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

static TSS: Lazy<TaskStateSegment> = Lazy::new(|| {
    let mut tss = TaskStateSegment::new();
    tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
        const STACK_SIZE: usize = 4096 * 5;
        static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE]; // should be a proper stack allocation

        let stack_start = VirtAddr::from_ptr(unsafe { addr_of!(STACK) });
        stack_start + STACK_SIZE
    };
    tss
});

static GDT: Lazy<(GlobalDescriptorTable, Selectors)> = Lazy::new(|| {
    let mut gdt = GlobalDescriptorTable::new();
    let kernel_code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
    let kernel_data_selector = gdt.add_entry(Descriptor::kernel_data_segment());
    let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
    let user_code_selector = gdt.add_entry(Descriptor::user_code_segment());
    let user_data_selector = gdt.add_entry(Descriptor::user_data_segment());
    (
        gdt,
        Selectors {
            kernel_code_selector,
            kernel_data_selector,
            tss_selector,
            user_code_selector,
            user_data_selector,
        },
    )
});

#[allow(dead_code)]
struct Selectors {
    kernel_code_selector: SegmentSelector,
    kernel_data_selector: SegmentSelector,
    tss_selector: SegmentSelector,
    user_code_selector: SegmentSelector,
    user_data_selector: SegmentSelector,
}

pub fn init() {
    GDT.0.load();
    unsafe {
        CS::set_reg(GDT.1.kernel_code_selector);
        DS::set_reg(GDT.1.kernel_data_selector);
        load_tss(GDT.1.tss_selector);
    }
}
