use core::arch::asm;
use core::arch::x86_64::_mm_pause;

fn main() {
    let res = syscall1(0, 17);
    let _ = syscall1(1, res as usize);
    loop {
        unsafe { _mm_pause() };
    }
}

fn syscall1(n: usize, arg: usize) -> isize {
    unsafe {
        let res: isize;
        asm! {
        "int 0x80",
        in("rax") n,
        in("rdi") arg,
        lateout("rax") res,
        }
        res
    }
}
