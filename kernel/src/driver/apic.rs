use acpi::platform::interrupt::Apic;
use alloc::alloc::Global;
use alloc::format;
use alloc::string::ToString;
use conquer_once::spin::OnceCell;
use spin::Mutex;
use x2apic::ioapic::{IoApic, IrqFlags, IrqMode, RedirectionTableEntry};
use x2apic::lapic::{xapic_base, LocalApic, LocalApicBuilder, TimerDivide, TimerMode};
use x86_64::instructions::port::Port;
use x86_64::structures::paging::{PageTableFlags, PhysFrame};
use x86_64::{PhysAddr, VirtAddr};

use crate::arch::idt::InterruptIndex;
use crate::mem::virt::{AllocationStrategy, MapAt};
use crate::mem::Size;
use crate::process::vmm;
use crate::Result;

pub static LAPIC: OnceCell<Mutex<LocalApic>> = OnceCell::uninit();

pub static KERNEL_LAPIC_ADDR: OnceCell<VirtAddr> = OnceCell::uninit();
pub static KERNEL_LAPIC_LEN: Size = Size::KiB(4); // 1 page
pub static KERNEL_IOAPIC_ADDR: OnceCell<VirtAddr> = OnceCell::uninit();
pub static KERNEL_IOAPIC_LEN: Size = Size::KiB(4); // 1 page

pub fn init(apic: Apic<Global>) -> Result<()> {
    disable_8259();

    let lapic_address = apic.local_apic_address;

    let lapic_id = init_lapic(lapic_address)?;

    for (i, io_apic) in apic.io_apics.iter().enumerate() {
        let ioapic_phys_addr = PhysAddr::try_new(io_apic.address as u64)
            .map_err(|e| format!("physical address {:#p} is not valid", e.0 as *const ()))?;
        let ioapic_phys_frame = PhysFrame::containing_address(ioapic_phys_addr);

        let ioapic_virtual_address: VirtAddr = vmm().allocate_memory_backed_vmobject(
            format!("ioapic{i}"),
            MapAt::Fixed(
                *KERNEL_IOAPIC_ADDR
                    .try_get()
                    .expect("kernel ioapic address should be initialized"),
            ),
            KERNEL_IOAPIC_LEN.bytes(),
            AllocationStrategy::MapNow(&[ioapic_phys_frame]),
            PageTableFlags::PRESENT
                | PageTableFlags::WRITABLE
                | PageTableFlags::NO_CACHE
                | PageTableFlags::NO_EXECUTE,
        )?;

        unsafe {
            let mut ioapic = IoApic::new(ioapic_virtual_address.as_u64());
            const OFFSET: u8 = 32;
            ioapic.init(OFFSET);
            for vector in 0..u8::MAX - OFFSET {
                let mut entry = RedirectionTableEntry::default();
                entry.set_mode(IrqMode::Fixed);
                entry.set_flags(IrqFlags::LEVEL_TRIGGERED | IrqFlags::LOW_ACTIVE);
                entry.set_vector(vector);
                entry.set_dest(u8::try_from(lapic_id).unwrap());

                ioapic.set_table_entry(vector, entry);
                ioapic.enable_irq(vector);
            }
        }
    }

    Ok(())
}

fn init_lapic(lapic_address: u64) -> Result<u32> {
    debug_assert_eq!(unsafe { xapic_base() }, lapic_address);
    let lapic_phys_addr = PhysAddr::try_new(lapic_address)
        .map_err(|e| format!("physical address {:#p} is not valid", e.0 as *const ()))?;
    let lapic_phys_frame = PhysFrame::containing_address(lapic_phys_addr);

    let lapic_virtual_address: VirtAddr = vmm().allocate_memory_backed_vmobject(
        "lapic".to_string(),
        MapAt::Fixed(
            *KERNEL_LAPIC_ADDR
                .try_get()
                .expect("kernel lapic address should be initialized"),
        ),
        KERNEL_LAPIC_LEN.bytes(),
        AllocationStrategy::MapNow(&[lapic_phys_frame]),
        PageTableFlags::PRESENT
            | PageTableFlags::WRITABLE
            | PageTableFlags::NO_CACHE
            | PageTableFlags::NO_EXECUTE,
    )?;

    let mut lapic = LocalApicBuilder::new()
        .timer_vector(InterruptIndex::Timer.into())
        .error_vector(InterruptIndex::LapicErr.into())
        .spurious_vector(InterruptIndex::Spurious.into())
        .set_xapic_base(lapic_virtual_address.as_u64())
        .timer_mode(TimerMode::Periodic)
        .timer_initial(312500)
        .timer_divide(TimerDivide::Div16)
        .build()?;

    unsafe {
        lapic.enable();
    }
    let id = unsafe { lapic.id() };
    LAPIC.init_once(move || Mutex::new(lapic));
    Ok(id)
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
