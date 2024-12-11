use crate::arch::syscall::syscall_handler_impl;
use crate::driver::apic::LAPIC;
use crate::driver::rtl8139::rtl8139_interrupt_handler;
use crate::process;
use crate::process::vmm;
use alloc::boxed::Box;
use conquer_once::spin::OnceCell;
use core::mem::transmute;
use core::pin::Pin;
use kernel_api::syscall::SYSCALL_INTERRUPT_INDEX;
use log::{info, warn};
use num_enum::IntoPrimitive;
use seq_macro::seq;
use spin::RwLock;
use x86_64::instructions::interrupts;
use x86_64::structures::idt::{
    Entry, InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode,
};
use x86_64::structures::paging::PageTableFlags;
use x86_64::PrivilegeLevel;

// needs to be pinned for safety guarantees in `::reload()`.
static IDT: OnceCell<RwLock<Pin<Box<InterruptDescriptorTable>>>> = OnceCell::uninit();

fn idt() -> &'static RwLock<Pin<Box<InterruptDescriptorTable>>> {
    IDT.get_or_init(|| RwLock::new(Box::pin(InterruptDescriptorTable::new())))
}

pub fn init() {
    let mut idt = idt().write();
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
    idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);
    unsafe {
        idt.double_fault
            .set_handler_fn(double_fault_handler)
            .set_stack_index(crate::gdt::DOUBLE_FAULT_IST_INDEX);
    }

    // set up catch all handlers for interrupts
    seq!(N in 32..=255 { // must be same as CUSTOM_INTERRUPT_RANGE
        idt[N].set_handler_fn(catch_all_handler::<N>);
    });

    // set up custom interrupts
    unsafe {
        idt[InterruptIndex::Syscall.as_usize()]
            .set_handler_fn(transmute::<
                *mut fn(),
                extern "x86-interrupt" fn(InterruptStackFrame),
            >(syscall_handler as *mut fn()))
            .set_privilege_level(PrivilegeLevel::Ring3)
            /*
            TODO: verify
            Now I might be completely wrong here, but since we use Rust, we should be safe
            with interrupts enabled during syscalls - as long as our preemptive multitasking
            works correctly. Everything that might be accessed falls under Rust's guarantees.
            As long as we have a separate stack for each syscall - even if we get preempted
            during one - we should be fine.
            */
            .disable_interrupts(false);
    }
    idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
    idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
    idt[InterruptIndex::LapicErr.as_usize()].set_handler_fn(lapic_err_interrupt_handler);
    idt[InterruptIndex::Spurious.as_usize()].set_handler_fn(spurious_interrupt_handler);
    idt[InterruptIndex::Rtc.as_usize()].set_handler_fn(rtc_handler);
    idt[InterruptIndex::Rtl8139.as_usize()].set_handler_fn(rtl8139_interrupt_handler);

    drop(idt); // unlock before loading
    reload();
}

pub fn reload() {
    let guard = idt().read();
    unsafe {
        // Safety: IDT is pinned
        guard.load_unsafe()
    };
}

pub fn next_free_interrupt_vector() -> Option<u8> {
    let idt = idt().read();
    (32..=255).find(|&i| idt[i as usize] == Entry::missing())
}

pub fn register_interrupt_handler(
    index: u8,
    handler: extern "x86-interrupt" fn(InterruptStackFrame),
) {
    let mut idt = idt().write();
    assert!(index >= 32, "invalid interrupt index");
    assert_eq!(
        idt[index as usize],
        Entry::missing(),
        "interrupt already registered"
    );
    idt[index as usize].set_handler_fn(handler);

    reload();
}

#[derive(Debug, Clone, Copy, IntoPrimitive)]
#[repr(u8)]
pub enum InterruptIndex {
    /// 32
    Timer = 0x20,
    /// 33
    Keyboard = 0x21,
    /// 49
    LapicErr = 0x31,
    /// 64
    IpiWake = 0x40,
    /// 65
    IpiTlb = 0x41,
    /// 66
    IpiSwitch = 0x42,
    /// 67
    IpiPit = 0x43,
    Syscall = SYSCALL_INTERRUPT_INDEX,
    Rtc = 0x82, // not something that we decide currently, TODO: disable entirely - we don't need it
    Rtl8139 = 0x83, // TODO: maybe summarize all network interrupts at some point
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

macro_rules! wrap {
    ($fn:ident => $w:ident) => {
        #[allow(clippy::missing_safety_doc)]
        #[naked]
        pub unsafe extern "sysv64" fn $w() {
            core::arch::naked_asm!(
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
                sym $fn
            );
        }
    };
}

wrap!(syscall_handler_impl => syscall_handler);

extern "x86-interrupt" fn catch_all_handler<const VECTOR: usize>(
    _stack_frame: InterruptStackFrame,
) {
    warn!("unhandled interrupt vector {VECTOR}");
    unsafe {
        end_of_interrupt();
    }
}

extern "x86-interrupt" fn rtc_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        end_of_interrupt();
    }
}

extern "x86-interrupt" fn divide_error_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: DIVIDE ERROR\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn lapic_err_interrupt_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: LAPIC ERROR\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn spurious_interrupt_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: SPURIOUS INTERRUPT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    info!(
        "encountered a general protection fault, error code {} =",
        error_code
    );
    info!("index: {}", (error_code >> 3) & ((1 << 14) - 1));
    info!("tbl: {}", (error_code >> 1) & 0b11);
    info!("e: {}", error_code & 1);

    panic!("EXCEPTION: GENERAL PROTECTION FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    info!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
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

extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: InterruptStackFrame) {
    let current_pid = *process::current().pid();
    let current_tid = process::current_thread().id();
    panic!(
        "EXCEPTION: INVALID OPCODE\ncurrent pid={},tid={}\n{:#?}",
        current_pid, current_tid, stack_frame
    );
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        end_of_interrupt();
    }

    // after the interrupt is handled, because we'll switch to another thread
    unsafe { process::reschedule() };
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    // FIXME: not all error codes can be handled like this

    let accessed_address = Cr2::read();

    let do_panic = || -> ! {
        panic!(
            "EXCEPTION: PAGE FAULT\nAccessed Address: {:?}\nError Code: {:?}\n{:#?}",
            accessed_address, error_code, stack_frame
        );
    };

    interrupts::enable();

    // IMPORTANT: From here, we need to be 100% thread safe!

    let vm_objects = vmm().vm_objects().read();
    let vm_object = vm_objects
        .iter()
        .find(|(_, vm_object)| vm_object.contains_addr(accessed_address))
        .map(|(_, vm_object)| vm_object);

    if vm_object.is_none() {
        do_panic();
    }

    let vm_object = vm_object.unwrap();
    if error_code.contains(PageFaultErrorCode::CAUSED_BY_WRITE)
        && !vm_object.flags().contains(PageTableFlags::WRITABLE)
    {
        do_panic();
    }

    let offset = (accessed_address.as_u64() - vm_object.addr().as_u64()) as usize;
    vm_object.prepare_for_access(offset).unwrap();
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use x86_64::instructions::port::Port;

    let mut port = Port::new(0x60);
    let _scancode: u8 = unsafe { port.read() };
    // TODO: put scancode into scancode queue
}

/// Notifies the LAPIC that the interrupt has been handled.
///
/// # Safety
/// This is unsafe since it writes to an LAPIC register.
#[inline]
pub unsafe fn end_of_interrupt() {
    LAPIC.get().unwrap().lock().end_of_interrupt();
}

#[cfg(feature = "kernel_test")]
mod tests {
    use kernel_test_framework::kernel_test;

    #[kernel_test]
    fn test_breakpoint_exception() {
        // invoke a breakpoint exception
        x86_64::instructions::interrupts::int3();
        // if this test returns that means that the interrupt handler is working
    }
}
