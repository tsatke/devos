use alloc::string::ToString;
use alloc::vec::Vec;

use conquer_once::spin::OnceCell;
use spin::{Mutex, MutexGuard};
use x86_64::structures::paging::{PageTableFlags, PhysFrame, Size4KiB};
use x86_64::PhysAddr;

use graphics::PrimitiveDrawing;
use pci::{BaseAddressRegister, PciStandardHeaderDevice};
use vga::{Color, FrameBuffer, Vga1280x800};

use crate::mem::virt::{AllocationStrategy, MapAt};
use crate::process::vmm;

static VGA: OnceCell<Mutex<Vga1280x800>> = OnceCell::uninit();

pub fn init() {
    if let Some(ctrl) = pci::devices()
        .find(|device| {
            matches!(
                device.class(),
                pci::PciDeviceClass::DisplayController(
                    pci::DisplaySubClass::VGACompatibleController
                )
            )
        })
        .map(|device| PciStandardHeaderDevice::new(device.clone()).unwrap())
    {
        let bar0 = ctrl.bar0();
        let (addr, size) = match bar0 {
            BaseAddressRegister::MemorySpace32(bar) => (bar.addr as u64, bar.size),
            BaseAddressRegister::MemorySpace64(bar) => (bar.addr, bar.size),
            _ => {
                panic!("VGA controller has no memory space BAR")
            }
        };

        let frames = (addr..addr + size as u64)
            .step_by(4096)
            .map(PhysAddr::new)
            .map(PhysFrame::<Size4KiB>::containing_address)
            .collect::<Vec<_>>();

        let vaddr = vmm()
            .allocate_memory_backed_vmobject(
                "framebuffer".to_string(),
                MapAt::Anywhere,
                size,
                AllocationStrategy::MapNow(frames),
                PageTableFlags::PRESENT
                    | PageTableFlags::WRITABLE
                    | PageTableFlags::NO_EXECUTE
                    | PageTableFlags::NO_CACHE
                    | PageTableFlags::WRITE_THROUGH,
            )
            .unwrap();

        VGA.init_once(|| {
            Mutex::new(Vga1280x800::new(unsafe {
                FrameBuffer::from_ptr(vaddr.as_mut_ptr::<u32>())
            }))
        });
    }
}

pub fn vga_initialized() -> bool {
    VGA.is_initialized()
}

pub fn lock() -> MutexGuard<'static, impl PrimitiveDrawing<Color>> {
    VGA.get().unwrap().lock()
}
