use alloc::string::ToString;
use alloc::{format, vec};

use conquer_once::spin::OnceCell;
use spin::Mutex;
use x2apic::lapic::{xapic_base, LocalApic, LocalApicBuilder, TimerDivide, TimerMode};
use x86_64::instructions::port::Port;
use x86_64::structures::paging::{PageTableFlags, PhysFrame};
use x86_64::{PhysAddr, VirtAddr};

use crate::arch::idt::InterruptIndex;
use crate::mem::virt::{AllocationStrategy, MapAt};
use crate::mem::Size;
use crate::process::vmm;
use crate::{serial_println, Result};

pub static LAPIC: OnceCell<Mutex<LocalApic>> = OnceCell::uninit();

pub static KERNEL_APIC_ADDR: OnceCell<VirtAddr> = OnceCell::uninit();
pub static KERNEL_APIC_LEN: Size = Size::KiB(4); // 1 page

pub fn init() -> Result<()> {
    disable_8259();

    let apic_physical_address: u64 = unsafe { xapic_base() };
    let apic_virtual_address: VirtAddr = vmm().allocate_memory_backed_vmobject(
        "apic".to_string(),
        MapAt::Fixed(
            *KERNEL_APIC_ADDR
                .get()
                .expect("KERNEL_APIC_ADDR not initialized"),
        ),
        KERNEL_APIC_LEN.bytes(),
        AllocationStrategy::MapNow(vec![PhysFrame::containing_address(
            PhysAddr::try_new(apic_physical_address)
                .map_err(|e| format!("physical address {:#p} is not valid", e.0 as *const ()))?,
        )]),
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
    LAPIC.init_once(move || Mutex::new(lapic));

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
