#![no_std]
#![no_main]

use core::slice::from_raw_parts_mut;
use limine::request::{
    FramebufferRequest, KernelFileRequest, RequestsEndMarker, RequestsStartMarker,
};
use limine::BaseRevision;
use log::{error, info};
use x86_64::instructions::hlt;

#[used]
#[unsafe(link_section = ".requests")]
static BASE_REVISION: BaseRevision = BaseRevision::new();

#[used]
#[unsafe(link_section = ".requests")]
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

#[used]
#[unsafe(link_section = ".requests_start_marker")]
static _START_MARKER: RequestsStartMarker = RequestsStartMarker::new();

#[used]
#[unsafe(link_section = ".requests_end_marker")]
static _END_MARKER: RequestsEndMarker = RequestsEndMarker::new();

#[used]
#[unsafe(link_section = ".requests")]
static KERNEL_FILE: KernelFileRequest = KernelFileRequest::new();

#[unsafe(export_name = "kernel_main")]
unsafe extern "C" fn main() -> ! {
    assert!(BASE_REVISION.is_supported());

    kernel::init();

    if let Some(framebuffer_response) = FRAMEBUFFER_REQUEST.get_response() {
        if let Some(framebuffer) = framebuffer_response.framebuffers().next() {
            let slice = unsafe {
                #[allow(clippy::cast_ptr_alignment)]
                from_raw_parts_mut(
                    framebuffer.addr().cast::<u32>(),
                    (framebuffer.pitch() * framebuffer.height() / 4) as usize,
                )
            };
            slice.fill(0x00_11_AA_11);

            for i in 0..100_u64 {
                // Calculate the pixel offset using the framebuffer information we obtained above.
                // We skip `i` scanlines (pitch is provided in bytes) and add `i * 4` to skip `i` pixels forward.
                let pixel_offset = i * framebuffer.pitch() + i * 4;

                // Write 0xFFFFFFFF to the provided pixel offset to fill it white.
                #[allow(clippy::cast_ptr_alignment)]
                unsafe {
                    *(framebuffer
                        .addr()
                        .add(usize::try_from(pixel_offset).expect("usize overflow"))
                        .cast::<u32>()) = 0xFFFF_FFFF;
                }
            }
        }
    }

    info!("reached end of kernel_main");
    loop {
        hlt();
    }
}

#[panic_handler]
fn rust_panic(info: &core::panic::PanicInfo) -> ! {
    let location = info.location().unwrap();
    error!(
        "kernel panicked at {}:{}:{}:\n{}",
        location.file(),
        location.line(),
        location.column(),
        info.message(),
    );
    loop {
        hlt();
    }
}
