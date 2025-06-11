use alloc::boxed::Box;
use core::arch::x86_64::_fxsave;

use raw_cpuid::CpuId;
use x86_64::registers::control::{Cr0, Cr0Flags, Cr4, Cr4Flags};

use crate::mem::heap::Heap;

pub fn init() {
    let cpuid = CpuId::new();
    let cpu_feature_info = cpuid.get_feature_info().expect("should have feature info");
    assert!(
        cpu_feature_info.has_sse(),
        "this cpu does not support sse, but it is required"
    );
    assert!(
        cpu_feature_info.has_sse2(),
        "this cpu does not support sse2, but it is required"
    );
    assert!(
        cpu_feature_info.has_fxsave_fxstor(),
        "this cpu does not support fxsave/fxrstor, but it is required"
    );

    // enable SSE and FXSAVE/FXRSTOR
    unsafe {
        Cr0::update(|cr0| {
            cr0.remove(Cr0Flags::EMULATE_COPROCESSOR);
            cr0.insert(Cr0Flags::MONITOR_COPROCESSOR);
        });
        Cr4::update(|cr4| {
            cr4.insert(Cr4Flags::OSFXSR);
            cr4.insert(Cr4Flags::OSXMMEXCPT_ENABLE);
        });
    }

    assert!(Heap::is_initialized());

    // FIXME: THIS CAN'T BE A KERNEL ALLOCATION!
    let fx_area_ptr = Box::into_raw(Box::new(FxArea { data: [0; 512] }));
    unsafe { _fxsave(fx_area_ptr.cast()) };
}

#[repr(C)]
struct FxArea {
    data: [u8; 512],
}
