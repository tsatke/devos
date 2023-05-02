use conquer_once::spin::OnceCell;
use spin::Mutex;
use x2apic::lapic::{xapic_base, LocalApic, LocalApicBuilder, TimerDivide, TimerMode};
use x86_64::instructions::port::Port;
use x86_64::structures::paging::Size4KiB;
use x86_64::structures::paging::{Page, PageTableFlags, PhysFrame};
use x86_64::{PhysAddr, VirtAddr};

use crate::arch::idt::InterruptIndex;
use crate::map_page;

pub static LAPIC: OnceCell<Mutex<LocalApic>> = OnceCell::uninit();

pub fn init() {
    disable_8259();

    let apic_physical_address: u64 = unsafe { xapic_base() };
    let apic_virtual_address: u64 = 0x2222_2222_0000; // TODO: make dynamic

    let apic_page = Page::containing_address(VirtAddr::new(apic_virtual_address));
    let frame = PhysFrame::containing_address(PhysAddr::new(apic_physical_address));
    map_page!(
        apic_page,
        frame,
        Size4KiB,
        PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_CACHE
    );

    let mut lapic = LocalApicBuilder::new()
        .timer_vector(InterruptIndex::Timer.into())
        .error_vector(InterruptIndex::LapicErr.into())
        .spurious_vector(InterruptIndex::Spurious.into())
        .set_xapic_base(apic_virtual_address)
        .timer_mode(TimerMode::Periodic)
        .timer_initial(312500)
        .timer_divide(TimerDivide::Div16)
        .build()
        .unwrap_or_else(|err| panic!("{}", err));

    unsafe {
        lapic.enable();
    }
    LAPIC.init_once(move || Mutex::new(lapic));
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
