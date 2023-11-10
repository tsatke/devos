use kernel_api::syscall::Errno;

pub type Result<T> = core::result::Result<T, Errno>;
