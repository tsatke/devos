use crate::process::syscall::dispatch_syscall;
use crate::serial_println;
use core::arch::asm;
use core::mem::transmute;
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin::Mutex;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use x86_64::PrivilegeLevel;

// "Remapped" PICS chosen as 32 to 47
pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: Mutex<ChainedPics> =
    Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.divide_error.set_handler_fn(divide_error_handler);
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt.overflow.set_handler_fn(overflow_handler);
        idt.general_protection_fault
            .set_handler_fn(general_protection_fault_handler);
        idt.stack_segment_fault
            .set_handler_fn(stack_segment_fault_handler);
        idt.segment_not_present
            .set_handler_fn(segment_not_present_fault_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(crate::gdt::DOUBLE_FAULT_IST_INDEX);
            idt[0x80]
                .set_handler_fn(transmute(syscall_handler as *mut fn()))
                .set_privilege_level(PrivilegeLevel::Ring3);
        }
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
        idt[46].set_handler_fn(ignore_handler);
        idt
    };
}

pub fn init() {
    IDT.load();
    unsafe {
        PICS.lock().initialize();
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

macro_rules! wrap {
    ($fn: ident => $w:ident) => {
        #[naked]
        pub unsafe extern "sysv64" fn $w() {
            asm!(
                "push rax",
                "push rcx",
                "push rdx",
                "push rsi",
                "push rdi",
                "push r8",
                "push r9",
                "push r10",
                "push r11",
                "mov rsi, rsp", // Arg #2: register list
                "mov rdi, rsp", // Arg #1: interupt frame
                "add rdi, 9 * 8",
                "call {}",
                "pop r11",
                "pop r10",
                "pop r9",
                "pop r8",
                "pop rdi",
                "pop rsi",
                "pop rdx",
                "pop rcx",
                "pop rax",
                "iretq",
                sym $fn,
                options(noreturn)
            );
        }
    };
}

wrap!(syscall_handler_impl => syscall_handler);

#[repr(align(8), C)]
#[derive(Debug, Clone, Copy, Default)]
struct SyscallRegisters {
    pub r11: usize,
    pub r10: usize,
    pub r9: usize,
    pub r8: usize,
    pub rdi: usize,
    pub rsi: usize,
    pub rdx: usize,
    pub rcx: usize,
    pub rax: usize,
}

extern "sysv64" fn syscall_handler_impl(
    _stack_frame: &mut InterruptStackFrame,
    regs: &mut SyscallRegisters,
) {
    // The registers order follow the System V ABI convention
    let n = regs.rax;
    let arg1 = regs.rdi;
    let arg2 = regs.rsi;
    let arg3 = regs.rdx;
    let arg4 = regs.r8;

    let res = dispatch_syscall(n, arg1, arg2, arg3, arg4);

    regs.rax = res as usize; // save result

    unsafe {
        PICS.lock().notify_end_of_interrupt(0x80);
    }
}

extern "x86-interrupt" fn divide_error_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: DIVIDE ERROR\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn ignore_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        PICS.lock().notify_end_of_interrupt(46);
    }
}

extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    serial_println!(
        "encountered a general protection fault, error code {} =",
        error_code
    );
    serial_println!("index: {}", (error_code >> 3) & ((1 << 14) - 1));
    serial_println!("tbl: {}", (error_code >> 1) & 0b11);
    serial_println!("e: {}", error_code & 1);

    panic!("EXCEPTION: GENERAL PROTECTION FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    serial_println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn overflow_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: OVERFLOW\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn stack_segment_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!(
        "EXCEPTION: STACK SEGMENT FAULT\nerror code: {}\n{:#?}",
        error_code, stack_frame
    );
}

extern "x86-interrupt" fn segment_not_present_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!(
        r#"EXCEPTION: SEGMENT NOT PRESENT FAULT
instruction pointer: {:p}
error code: {} ({:#b})
external: {}
table[index]: {}[{}]
{:#?}"#,
        stack_frame.instruction_pointer.as_u64() as *const u8,
        error_code,
        error_code,
        (error_code & 1) == 1,
        match (error_code & 0b110) >> 1 {
            0b00 => "GDT",
            0b01 => "IDT",
            0b10 => "LDT",
            0b11 => "IDT",
            _ => "unknown",
        },
        ((error_code & ((1 << 14) - 1)) >> 3),
        stack_frame
    );
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // TODO: notify timer and scheduler

    unsafe {
        end_of_interrupt(InterruptIndex::Timer);
    }
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    panic!(
        "EXCEPTION: PAGE FAULT\nAccessed Address: {:?}\nError Code: {:?}\n{:#?}",
        Cr2::read(),
        error_code,
        stack_frame
    );
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use x86_64::instructions::port::Port;

    let mut port = Port::new(0x60);
    let _scancode: u8 = unsafe { port.read() };
    // TODO: put scancode into scancode queue

    unsafe {
        end_of_interrupt(InterruptIndex::Keyboard);
    }
}

#[inline]
unsafe fn end_of_interrupt(which: InterruptIndex) {
    PICS.lock().notify_end_of_interrupt(which.as_u8());
}

#[cfg(test)]
mod tests {
    #[test_case]
    fn test_breakpoint_exception() {
        // invoke a breakpoint exception
        x86_64::instructions::interrupts::int3();
        // if this test returns that means that the interrupt handler is working
    }
}
