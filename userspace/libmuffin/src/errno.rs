use core::cell::Cell;

use libc::c_int;

#[thread_local]
static ERRNO: Cell<c_int> = Cell::new(0xAABB_CCDD_u32 as i32);

#[unsafe(no_mangle)]
pub unsafe extern "C" fn errno_location() -> *mut c_int {
    ERRNO.as_ptr()
}

pub(crate) fn set_errno<T: Into<c_int>>(value: T) {
    ERRNO.set(value.into());
}
