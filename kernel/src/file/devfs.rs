use core::fmt::Write;

use conquer_once::spin::OnceCell;
use kernel_devfs::{ArcLockedDevFs, Serial};
use kernel_vfs::path::AbsolutePath;

use crate::serial_print;

static DEVFS: OnceCell<ArcLockedDevFs> = OnceCell::uninit();

#[must_use]
pub fn devfs() -> &'static ArcLockedDevFs {
    DEVFS.get().expect("devfs should be initialized")
}

pub fn init() {
    let devfs = ArcLockedDevFs::new();
    {
        let mut guard = devfs.write();
        guard
            .register_file(AbsolutePath::try_new("/serial").unwrap(), || {
                Ok(Serial::<SerialWrite>::default())
            })
            .expect("should be able to register serial file");
    }
    DEVFS.init_once(|| devfs);
}

#[derive(Default)]
struct SerialWrite;

impl Write for SerialWrite {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        serial_print!("{s}");
        Ok(())
    }
}
