use crate::arch::{gdt, idt};
use crate::limine::MP_REQUEST;
use crate::smp::context::ExecutionContext;
use alloc::boxed::Box;
use log::{debug, info};
use x86_64::instructions::hlt;
use x86_64::registers::control::{Cr3, Cr3Flags};
use x86_64::registers::model_specific::GsBase;
use x86_64::structures::paging::PhysFrame;
use x86_64::{PhysAddr, VirtAddr};

pub mod context;

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

    debug!("read cr3_val: {:#x}", cr3_val);

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

    unsafe {
        let flags = Cr3Flags::from_bits_truncate(cpu.extra);
        Cr3::write(
            PhysFrame::containing_address(PhysAddr::new(cpu.extra)),
            flags,
        );
    }

    gdt::init();
    idt::init();

    {
        let ctx = ExecutionContext::from(cpu);
        let addr = VirtAddr::from_ptr(Box::into_raw(Box::new(ctx)));
        GsBase::write(addr);
    }

    let ctx = ExecutionContext::load();
    info!("cpu {} initialized", ctx.cpu_id());

    loop {
        hlt();
    }
}
