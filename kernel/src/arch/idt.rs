use crate::arch::gdt;
use crate::mcore::context::ExecutionContext;
use log::error;
use x86_64::instructions::{hlt, interrupts};
use x86_64::registers::control::Cr2;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    /// 32
    Timer = 0x20,
    /// 49
    LapicErr = 0x31,
    /// 255
    Spurious = 0xff,
}

impl InterruptIndex {
    pub fn as_usize(self) -> usize {
        self as usize
    }

    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

pub fn create_idt() -> InterruptDescriptorTable {
    let mut idt = InterruptDescriptorTable::new();

    unsafe {
        idt.double_fault
            .set_handler_fn(double_fault_handler)
            .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        idt.page_fault
            .set_handler_fn(page_fault_handler)
            .set_stack_index(gdt::PAGE_FAULT_IST_INDEX);
    }
    idt.general_protection_fault
        .set_handler_fn(general_protection_fault_handler);
    idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);
    idt.invalid_tss.set_handler_fn(invalid_tss_handler);
    idt.segment_not_present
        .set_handler_fn(segment_not_present_handler);
    idt.stack_segment_fault
        .set_handler_fn(stack_segment_fault_handler);

    idt[InterruptIndex::Timer.as_u8()].set_handler_fn(timer_interrupt_handler);
    idt[InterruptIndex::LapicErr.as_u8()].set_handler_fn(lapic_err_interrupt_handler);
    idt[InterruptIndex::Spurious.as_u8()].set_handler_fn(spurious_interrupt_handler);

    idt
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        end_of_interrupt();
    }

    let ctx = ExecutionContext::load();
    unsafe {
        ctx.scheduler_mut().reschedule();
    }
}

extern "x86-interrupt" fn lapic_err_interrupt_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: LAPIC ERROR\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn spurious_interrupt_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: SPURIOUS INTERRUPT\n{:#?}", stack_frame);
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

extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: INVALID OPCODE:\n{stack_frame:#?}");
}

extern "x86-interrupt" fn invalid_tss_handler(stack_frame: InterruptStackFrame, error_code: u64) {
    panic!("EXCEPTION: INVALID TSS:\nerror code: {error_code:#X}\n{stack_frame:#?}");
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    let accessed_address = Cr2::read().ok();

    // if we know the address...
    if let Some(addr) = accessed_address {
        // ...and we have initialized multitasking...
        if let Some(ctx) = ExecutionContext::try_load() {
            let task = ctx.current_task();
            // ...and the current task has stack...
            if let Some(stack) = task.stack() {
                // ...then the accessed address must not be within the guard page of the stack,
                // otherwise we have a stack overflow...
                if stack.guard_page().contains(addr) {
                    error!(
                        "STACK OVERFLOW DETECTED in process '{}' task '{}', terminating...",
                        task.process().name(),
                        task.name(),
                    );
                }

                // ...in which case we mark the task for termination...
                task.set_should_terminate(true);
                // ...and halt, waiting for the scheduler to terminate the task
                interrupts::enable();
                loop {
                    hlt();
                }
            }
        }
    }

    panic!(
        "EXCEPTION: PAGE FAULT:\naccessed address: {accessed_address:?}\nerror code: {error_code:#X}\n{stack_frame:#?}"
    );
}

extern "x86-interrupt" fn segment_not_present_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!("EXCEPTION: SEGMENT NOT PRESENT:\nerror code: {error_code:#X}\n{stack_frame:#?}");
}

extern "x86-interrupt" fn stack_segment_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!("EXCEPTION: STACK SEGMENT FAULT:\nerror code: {error_code:#X}\n{stack_frame:#?}");
}

/// Notifies the LAPIC that the interrupt has been handled.
///
/// # Safety
/// This is unsafe since it writes to an LAPIC register.
#[inline]
pub unsafe fn end_of_interrupt() {
    let ctx = ExecutionContext::load();
    unsafe { ctx.lapic().lock().end_of_interrupt() };
}
