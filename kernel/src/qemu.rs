use x86_64::instructions::port::Port;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit(exit_code: ExitCode) -> ! {
    let mut port = Port::new(0xf4);
    unsafe {
        port.write(exit_code as u32);
    }
    unreachable!()
}
