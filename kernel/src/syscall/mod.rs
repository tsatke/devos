use log::info;

pub fn dispatch_syscall(
    n: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
    arg5: usize,
    arg6: usize,
) -> usize {
    info!("syscall: {n} {arg1} {arg2} {arg3} {arg4} {arg5} {arg6}");
    0
}
