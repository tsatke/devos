use crate::arch::gdt::create_gdt_and_tss;
use crate::arch::idt::create_idt;
use crate::limine::MP_REQUEST;
use crate::mcore::context::ExecutionContext;
use crate::mcore::mtask::process::Process;
use crate::mcore::mtask::task::Task;
use alloc::boxed::Box;
use core::ffi::c_void;
use core::ptr;
use log::{debug, info};
use x86_64::instructions::hlt;
use x86_64::instructions::segmentation::{CS, DS};
use x86_64::instructions::tables::load_tss;
use x86_64::registers::control::{Cr3, Cr3Flags};
use x86_64::registers::model_specific::KernelGsBase;
use x86_64::registers::segmentation::Segment;
use x86_64::structures::paging::PhysFrame;
use x86_64::{PhysAddr, VirtAddr};

pub mod context;
pub mod mtask;

#[allow(clippy::missing_panics_doc)]
pub fn start() -> ! {
    let resp = unsafe {
        #[allow(static_mut_refs)] // we need this to set the `extra` field in the CPU structs
        MP_REQUEST.get_response_mut()
    }
    .unwrap();

    let cr3_val = {
        let (frame, flags) = Cr3::read();
        frame.start_address().as_u64() | flags.bits()
    };

    debug!("read cr3_val: {cr3_val:#x}");

    // set the extra field in the CPU structs to the CR3 value
    resp.cpus_mut().iter_mut().for_each(|cpu| {
        cpu.extra = cr3_val;
    });

    // then call the `cpu_init` function on each CPU (no-op on bootstrap CPU)
    resp.cpus().iter().for_each(|cpu| {
        cpu.goto_address.write(cpu_init);
    });

    // then call the `cpu_init` function on the bootstrap CPU
    unsafe { cpu_init(resp.cpus()[0]) }
}

unsafe extern "C" fn cpu_init(cpu: &limine::mp::Cpu) -> ! {
    debug!("booting cpu {} with argument {}", cpu.id, cpu.extra);

    // set the memory mapping that we got as a parameter
    unsafe {
        let flags = Cr3Flags::from_bits_truncate(cpu.extra);
        Cr3::write(
            PhysFrame::containing_address(PhysAddr::new(cpu.extra)),
            flags,
        );
    }

    // set up the GDT
    let (gdt, sel) = create_gdt_and_tss();
    let gdt = Box::leak(Box::new(gdt));
    gdt.load();
    unsafe {
        CS::set_reg(sel.kernel_code);
        DS::set_reg(sel.kernel_data);
        load_tss(sel.tss);
    }

    // set up the IDT
    let idt = create_idt();
    let idt = Box::leak(Box::new(idt));
    idt.load();

    // create the execution context for the CPU and store it
    {
        let ctx = ExecutionContext::new(cpu, gdt, idt);
        let addr = VirtAddr::from_ptr(Box::leak(Box::new(ctx)));
        KernelGsBase::write(addr);
    }

    // load it back and print a message
    let ctx = ExecutionContext::load();
    info!("cpu {} initialized", ctx.cpu_id());

    let new_task = Task::create_new(Process::root(), enter_task, ptr::null_mut()).unwrap();
    unsafe {
        ctx.scheduler().enqueue(new_task);
        ctx.scheduler().reschedule();
    }

    info!("back in the initial task, halting...");

    loop {
        hlt();
    }
}

extern "C" fn enter_task(arg: *mut c_void) {
    info!("hello from task with arg {arg:p}");

    unsafe {
        ExecutionContext::load().scheduler().reschedule();
    }

    unreachable!("with the current implementation, we shouldn't get here");
}
