use crate::arch::gdt;
use alloc::boxed::Box;
use conquer_once::spin::Lazy;
use core::pin::Pin;
use spin::RwLock;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

static IDT: Lazy<RwLock<Pin<Box<InterruptDescriptorTable>>>> = Lazy::new(|| {
    let mut idt = InterruptDescriptorTable::new();

    unsafe {
        idt.double_fault
            .set_handler_fn(double_fault_handler)
            .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
    }
    idt.general_protection_fault
        .set_handler_fn(general_protection_fault_handler);
    idt.segment_not_present
        .set_handler_fn(segment_not_present_handler);

    let idt = Box::pin(idt);
    let idt = RwLock::new(idt);
    idt
});

fn reload_idt() {
    unsafe { IDT.read().load_unsafe() };
}

pub fn init() {
    reload_idt();
}

extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, _: u64) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT:\n{stack_frame:#?}");
}

extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!("EXCEPTION: GENERAL PROTECTION FAULT:\nerror code: {error_code:#X}\n{stack_frame:#?}");
}

extern "x86-interrupt" fn segment_not_present_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!("EXCEPTION: SEGMENT NOT PRESENT:\nerror code: {error_code:#X}\n{stack_frame:#?}");
}
