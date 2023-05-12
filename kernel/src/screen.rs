use bootloader_api::info::FrameBufferInfo;
use conquer_once::spin::OnceCell;
use graphics::PrimitiveDrawing;
use spin::{Mutex, MutexGuard};
use vga::{Color, FrameBuffer, Vga1280x800};

static VGA: OnceCell<Mutex<Vga1280x800>> = OnceCell::uninit();

pub fn init(frame_buffer_start: *const u8, _info: FrameBufferInfo) {
    VGA.init_once(|| {
        Mutex::new(Vga1280x800::new(unsafe {
            FrameBuffer::from_ptr(frame_buffer_start as *mut u32)
        }))
    });
}

pub fn lock() -> MutexGuard<'static, impl PrimitiveDrawing<Color>> {
    VGA.get().unwrap().lock()
}
