use alloc::boxed::Box;
use alloc::vec;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::{PrivilegeLevel, VirtAddr};

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;
pub const PAGE_FAULT_IST_INDEX: u16 = 1;

fn create_tss() -> TaskStateSegment {
    let mut tss = TaskStateSegment::new();
    tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = allocate_ist_stack();
    tss.interrupt_stack_table[PAGE_FAULT_IST_INDEX as usize] = allocate_ist_stack();
    tss
}

fn allocate_ist_stack() -> VirtAddr {
    const STACK_SIZE: usize = 4096 * 5;
    let stack = Box::into_raw(vec![0_u8; STACK_SIZE].into_boxed_slice());

    let stack_start = VirtAddr::from_ptr(stack);
    stack_start + (STACK_SIZE as u64)
}

#[allow(dead_code)]
pub struct Selectors {
    pub kernel_code: SegmentSelector,
    pub kernel_data: SegmentSelector,
    pub tss: SegmentSelector,
    pub user_code: SegmentSelector,
    pub user_data: SegmentSelector,
}

pub fn create_gdt_and_tss() -> (GlobalDescriptorTable, Selectors) {
    let mut gdt = GlobalDescriptorTable::new();
    let kernel_code = gdt.append(Descriptor::kernel_code_segment());
    let kernel_data = gdt.append(Descriptor::kernel_data_segment());

    let tss = Box::leak(Box::new(create_tss()));
    let tss = gdt.append(Descriptor::tss_segment(tss));
    let mut user_code = gdt.append(Descriptor::user_code_segment());
    user_code.set_rpl(PrivilegeLevel::Ring3);
    let mut user_data = gdt.append(Descriptor::user_data_segment());
    user_data.set_rpl(PrivilegeLevel::Ring3);
    (
        gdt,
        Selectors {
            kernel_code,
            kernel_data,
            tss,
            user_code,
            user_data,
        },
    )
}
