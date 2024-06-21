use alloc::format;

use crate::syscall::sys_write;

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    let text = format!("{}", args);
    let _ = sys_write(0, text.as_bytes()); // FIXME: posix mandates that stdout is 1, not 0
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::print::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($fmt:expr) => ($crate::print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::print!(concat!($fmt, "\n"), $($arg)*));
}
