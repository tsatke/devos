use core::arch::asm;
use x86_64::registers::rflags::RFlags;
use x86_64::registers::segmentation::SegmentSelector;
use x86_64::VirtAddr;

pub struct IretqFrame {
    pub stack_segment: SegmentSelector,
    pub stack_pointer: VirtAddr,
    pub rflags: RFlags,
    pub code_segment: SegmentSelector,
    pub instruction_pointer: VirtAddr,
}

impl IretqFrame {
    pub unsafe fn iretq(self) -> ! {
        asm!(
            "push {stack_segment}",
            "push {stack_pointer}",
            "push {rflags}",
            "push {code_segment}",
            "push {instruction_pointer}",
            "iretq",
            stack_segment = in(reg) self.stack_segment.0,
            stack_pointer = in(reg) self.stack_pointer.as_u64(),
            rflags = in(reg) self.rflags.bits(),
            code_segment = in(reg) self.code_segment.0,
            instruction_pointer = in(reg) self.instruction_pointer.as_u64(),
            options(noreturn),
        )
    }
}
