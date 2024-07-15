use alloc::format;
use alloc::string::ToString;

use conquer_once::spin::OnceCell;
use x2apic::lapic::{LocalApic, LocalApicBuilder, TimerDivide, TimerMode, xapic_base};
use x86_64::{PhysAddr, VirtAddr};
use x86_64::instructions::port::Port;
use x86_64::structures::paging::{PageTableFlags, PhysFrame};

use crate::{Result, serial_println};
use crate::arch::idt::InterruptIndex;
use crate::mem::Size;
use crate::mem::virt::{AllocationStrategy, MapAt};
use crate::process::vmm;

static mut LAPIC: Option<LocalApic> = None;

pub static KERNEL_APIC_ADDR: OnceCell<VirtAddr> = OnceCell::uninit();
pub static KERNEL_APIC_LEN: Size = Size::KiB(4); // 1 page

pub unsafe fn lapic() -> &'static mut LocalApic {
    LAPIC.as_mut().expect("LAPIC not initialized")
}

pub fn init() -> Result<()> {
    disable_8259();

    let apic_phys_addr = PhysAddr::try_new(unsafe { xapic_base() })
        .map_err(|e| format!("physical address {:#p} is not valid", e.0 as *const ()))?;
    let apic_phys_frame = PhysFrame::containing_address(apic_phys_addr);

    let apic_virtual_address: VirtAddr = vmm().allocate_memory_backed_vmobject(
        "apic".to_string(),
        MapAt::Fixed(
            *KERNEL_APIC_ADDR
                .get()
                .expect("KERNEL_APIC_ADDR not initialized"),
        ),
        KERNEL_APIC_LEN.bytes(),
        AllocationStrategy::MapNow(&[apic_phys_frame]),
        PageTableFlags::PRESENT
            | PageTableFlags::WRITABLE
            | PageTableFlags::NO_CACHE
            | PageTableFlags::NO_EXECUTE,
    )?;
    serial_println!("apic_virtual_address: {:#p}", apic_virtual_address);

    let mut lapic = LocalApicBuilder::new()
        .timer_vector(InterruptIndex::Timer.into())
        .error_vector(InterruptIndex::LapicErr.into())
        .spurious_vector(InterruptIndex::Spurious.into())
        .set_xapic_base(apic_virtual_address.as_u64())
        .timer_mode(TimerMode::Periodic)
        .timer_initial(312500)
        .timer_divide(TimerDivide::Div16)
        .build()?;

    unsafe {
        lapic.enable();
    }
    unsafe { LAPIC = Some(lapic) };

    Ok(())
}

fn disable_8259() {
    unsafe {
        // Disable 8259 immediately, thanks kennystrawnmusic

        let mut cmd_8259a = Port::<u8>::new(0x20);
        let mut data_8259a = Port::<u8>::new(0x21);
        let mut cmd_8259b = Port::<u8>::new(0xa0);
        let mut data_8259b = Port::<u8>::new(0xa1);

        let mut spin_port = Port::<u8>::new(0x80);
        let mut spin = || spin_port.write(0);

        cmd_8259a.write(0x11);
        cmd_8259b.write(0x11);
        spin();

        data_8259a.write(0xf8);
        data_8259b.write(0xff);
        spin();

        data_8259a.write(0b100);
        spin();

        data_8259b.write(0b10);
        spin();

        data_8259a.write(0x1);
        data_8259b.write(0x1);
        spin();

        data_8259a.write(u8::MAX);
        data_8259b.write(u8::MAX);
    };
}
