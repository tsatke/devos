#![feature(start)]
#![no_std]

#[start]
fn start(_argc: isize, _args: *const *const u8) -> isize {
    main();
    7
}

fn main() {}

#[cfg(not(test))]
#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
