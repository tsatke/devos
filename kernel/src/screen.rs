use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::sync::Arc;
use alloc::vec::Vec;

use conquer_once::spin::OnceCell;
use spin::{Mutex, MutexGuard, RwLock};
use x86_64::structures::paging::{PageTableFlags, PhysFrame, Size4KiB};
use x86_64::{PhysAddr, VirtAddr};

use graphics::PrimitiveDrawing;
use pci::PciStandardHeaderDevice;
use vga::{Color, FrameBuffer, Vga1280x800};

use crate::mem::virt::{AllocationStrategy, MemoryBackedVmObject, PmObject};
use crate::process;

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
        let (addr, size) = if let Some(bar) = bar0.memory_space_32() {
            (bar.addr as u64, bar.size)
        } else if let Some(bar) = bar0.memory_space_64() {
            (bar.addr, bar.size)
        } else {
            panic!("VGA controller has no memory space BAR")
        };

        let frames = (addr..addr + size as u64)
            .step_by(4096)
            .map(PhysAddr::new)
            .map(PhysFrame::<Size4KiB>::containing_address)
            .collect::<Vec<_>>();

        let vaddr = VirtAddr::new(0x1111_0000_0000);
        let pmo = PmObject::new(AllocationStrategy::AllocateNow, frames);
        let vmo = MemoryBackedVmObject::new(
            "framebuffer".to_string(),
            Arc::new(RwLock::new(pmo)),
            AllocationStrategy::AllocateNow,
            vaddr,
            size,
            PageTableFlags::PRESENT
                | PageTableFlags::WRITABLE
                | PageTableFlags::NO_EXECUTE
                | PageTableFlags::NO_CACHE
                | PageTableFlags::WRITE_THROUGH,
        );
        vmo.map_pages().unwrap();
        let process = process::current();
        process.vm_objects().write().insert(vaddr, Box::new(vmo));

        VGA.init_once(|| {
            Mutex::new(Vga1280x800::new(unsafe {
                FrameBuffer::from_ptr(vaddr.as_mut_ptr::<u32>())
            }))
        });
    }
}

pub fn is_initialized() -> bool {
    VGA.is_initialized()
}

pub fn lock() -> MutexGuard<'static, impl PrimitiveDrawing<Color>> {
    VGA.get().unwrap().lock()
}
