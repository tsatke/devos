use crate::sys_types::{pid_t, pthread_attr_t, size_t};
use crate::time::timespec;
use core::ffi::{c_char, c_int, c_void};
use muffin_libc_spec_comment::posix_spec;

pub type sig_atomic_t = u64;

pub type sigset_t = u64;

#[repr(C)]
pub struct sigevent {
    pub sigev_notify: c_int,
    pub sigev_signo: c_int,
    pub sigev_value: sigval,
    pub sigev_notify_function: extern "C" fn(sigval),
    pub sigev_notify_attributes: *const pthread_attr_t,
}

#[repr(C)]
pub union sigval {
    pub sival_int: c_int,
    pub sival_ptr: *const c_void,
}

pub const SIGEV_NONE: c_int = 1 << 0;
pub const SIGEV_SIGNAL: c_int = 1 << 1;
pub const SIGEV_THREAD: c_int = 1 << 2;

pub const SIGABRT: c_int = 0;
pub const SIGALRM: c_int = 1;
pub const SIGBUS: c_int = 2;
pub const SIGCHLD: c_int = 3;
pub const SIGCONT: c_int = 4;
pub const SIGFPE: c_int = 5;
pub const SIGHUP: c_int = 6;
pub const SIGILL: c_int = 7;
pub const SIGINT: c_int = 8;
pub const SIGKILL: c_int = 9;
pub const SIGPIPE: c_int = 10;
pub const SIGQUIT: c_int = 11;
pub const SIGSEGV: c_int = 12;
pub const SIGSTOP: c_int = 13;
pub const SIGTERM: c_int = 14;
pub const SIGTSTP: c_int = 15;
pub const SIGTTIN: c_int = 16;
pub const SIGTTOU: c_int = 17;
pub const SIGUSR1: c_int = 18;
pub const SIGUSR2: c_int = 19;
pub const SIGWINCH: c_int = 20;
pub const SIGSYS: c_int = 21;
pub const SIGTRAP: c_int = 22;
pub const SIGURG: c_int = 23;
pub const SIGVTALRM: c_int = 24;
pub const SIGXCPU: c_int = 25;
pub const SIGXFSZ: c_int = 26;

#[repr(C)]
pub struct sigaction {
    pub sa_handler: extern "C" fn(c_int),
    pub sa_mask: sigset_t,
    pub sa_flags: c_int,
    pub sa_sigaction: extern "C" fn(c_int, *const siginfo_t, *const c_void),
}

pub const SIG_BLOCK: c_int = 1 << 0;
pub const SIG_UNBLOCK: c_int = 1 << 1;
pub const SIG_SETMASK: c_int = 1 << 2;

pub const SA_NOCLDSTOP: c_int = 1 << 0;
pub const SA_ONSTACK: c_int = 1 << 1;
pub const SA_RESETHAND: c_int = 1 << 2;
pub const SA_RESTART: c_int = 1 << 3;
pub const SA_SIGINFO: c_int = 1 << 4;
pub const SA_NOCLDWAIT: c_int = 1 << 5;
pub const SA_NODEFER: c_int = 1 << 6;
pub const SS_ONSTACK: c_int = 1 << 7;
pub const SS_DISABLE: c_int = 1 << 8;
pub const MINSIGSTKSZ: c_int = 1 << 9;
pub const SIGSTKSZ: c_int = 1 << 10;

pub type mcontext_t = u64;

#[repr(C)]
pub struct ucontext_t {
    pub uc_link: *const ucontext_t,
    pub uc_sigmask: sigset_t,
    pub uc_stack: stack_t,
    pub uc_mcontext: mcontext_t,
}

#[repr(C)]
pub struct stack_t {
    pub ss_sp: *const c_void,
    pub ss_size: size_t,
    pub ss_flags: c_int,
}

#[repr(C)]
pub struct siginfo_t {
    pub si_signo: c_int,
    pub si_code: c_int,
    pub si_errno: c_int,
    pub si_pid: c_int,
    pub si_uid: c_int,
    pub si_addr: *const c_void,
    pub si_status: c_int,
    pub si_value: sigval,
}

pub const ILL_ILLOPC: c_int = 1;
pub const ILL_ILLOPN: c_int = 2;
pub const ILL_ILLADR: c_int = 3;
pub const ILL_ILLTRP: c_int = 4;
pub const ILL_PRVOPC: c_int = 5;
pub const ILL_PRVREG: c_int = 6;
pub const ILL_COPROC: c_int = 7;
pub const ILL_BADSTK: c_int = 8;

pub const FPE_INTDIV: c_int = 1;
pub const FPE_INTOVF: c_int = 2;
pub const FPE_FLTDIV: c_int = 3;
pub const FPE_FLTOVF: c_int = 4;
pub const FPE_FLTUND: c_int = 5;
pub const FPE_FLTRES: c_int = 6;
pub const FPE_FLTINV: c_int = 7;
pub const FPE_FLTSUB: c_int = 8;

pub const SEGV_MAPERR: c_int = 1;
pub const SEGV_ACCERR: c_int = 2;

pub const BUS_ADRALN: c_int = 1;
pub const BUS_ADRERR: c_int = 2;
pub const BUS_OBJERR: c_int = 3;

pub const TRAP_BRKPT: c_int = 1;
pub const TRAP_TRACE: c_int = 2;

pub const CLD_EXITED: c_int = 1;
pub const CLD_KILLED: c_int = 2;
pub const CLD_DUMPED: c_int = 3;
pub const CLD_TRAPPED: c_int = 4;
pub const CLD_STOPPED: c_int = 5;
pub const CLD_CONTINUED: c_int = 6;

pub const SI_USER: c_int = 256 + 1;
pub const SI_QUEUE: c_int = 256 + 2;
pub const SI_TIMER: c_int = 256 + 3;
pub const SI_ASYNCIO: c_int = 256 + 4;
pub const SI_MESGQ: c_int = 256 + 5;

#[posix_spec("functions/kill.html")]
#[unsafe(no_mangle)]
pub extern "C" fn kill(_pid: pid_t, _sig: c_int) -> c_int {
    todo!()
}

#[posix_spec("functions/killpg.html")]
#[unsafe(no_mangle)]
pub extern "C" fn killpg(_pgrp: pid_t, _sig: c_int) -> c_int {
    todo!()
}

#[posix_spec("functions/psiginfo.html")]
#[unsafe(no_mangle)]
pub extern "C" fn psiginfo(_info: *const siginfo_t, _msg: *const c_char) {
    todo!()
}

#[posix_spec("functions/psignal.html")]
#[unsafe(no_mangle)]
pub extern "C" fn psignal(_sig: c_int, _msg: *const c_char) {
    todo!()
}

#[posix_spec("functions/pthread_kill.html")]
#[unsafe(no_mangle)]
pub extern "C" fn pthread_kill(_thread: pthread_attr_t, _sig: c_int) -> c_int {
    todo!()
}

#[posix_spec("functions/pthread_sigmask.html")]
#[unsafe(no_mangle)]
pub extern "C" fn pthread_sigmask(_how: c_int, _set: &sigset_t, _oldset: &mut sigset_t) -> c_int {
    todo!()
}

#[posix_spec("functions/raise.html")]
#[unsafe(no_mangle)]
pub extern "C" fn raise(_sig: c_int) -> c_int {
    todo!()
}

#[posix_spec("functions/sig2str.html")]
#[unsafe(no_mangle)]
pub extern "C" fn sig2str(_sig: c_int, _str: &mut c_char) -> c_int {
    todo!()
}

#[posix_spec("functions/sigaction.html")]
#[unsafe(no_mangle)]
pub extern "C" fn sigaction(_sig: c_int, _act: &sigaction, _oact: &mut sigaction) -> c_int {
    todo!()
}

#[posix_spec("functions/sigaddset.html")]
#[unsafe(no_mangle)]
pub extern "C" fn sigaddset(_set: &mut sigset_t, _sig: c_int) -> c_int {
    todo!()
}

#[posix_spec("functions/sigaltstack.html")]
#[unsafe(no_mangle)]
pub extern "C" fn sigaltstack(_ss: &stack_t, _oss: &mut stack_t) -> c_int {
    todo!()
}

#[posix_spec("functions/sigdelset.html")]
#[unsafe(no_mangle)]
pub extern "C" fn sigdelset(_set: &mut sigset_t, _sig: c_int) -> c_int {
    todo!()
}

#[posix_spec("functions/sigemptyset.html")]
#[unsafe(no_mangle)]
pub extern "C" fn sigemptyset(_set: &mut sigset_t) -> c_int {
    todo!()
}

#[posix_spec("functions/sigfillset.html")]
#[unsafe(no_mangle)]
pub extern "C" fn sigfillset(_set: &mut sigset_t) -> c_int {
    todo!()
}

#[posix_spec("functions/sigismember.html")]
#[unsafe(no_mangle)]
pub extern "C" fn sigismember(_set: &sigset_t, _sig: c_int) -> c_int {
    todo!()
}

#[posix_spec("functions/signal.html")]
#[unsafe(no_mangle)]
pub extern "C" fn signal(_sig: c_int, _handler: extern "C" fn(c_int)) -> extern "C" fn(c_int) {
    todo!()
}

#[posix_spec("functions/sigpending.html")]
#[unsafe(no_mangle)]
pub extern "C" fn sigpending(_set: &mut sigset_t) -> c_int {
    todo!()
}

#[posix_spec("functions/sigprocmask.html")]
#[unsafe(no_mangle)]
pub extern "C" fn sigprocmask(_how: c_int, _set: &sigset_t, _oldset: &mut sigset_t) -> c_int {
    todo!()
}

#[posix_spec("functions/sigqueue.html")]
#[unsafe(no_mangle)]
pub extern "C" fn sigqueue(_pid: pid_t, _sig: c_int, _value: sigval) -> c_int {
    todo!()
}

#[posix_spec("functions/sigsuspend.html")]
#[unsafe(no_mangle)]
pub extern "C" fn sigsuspend(_set: &sigset_t) -> c_int {
    todo!()
}

#[posix_spec("functions/sigtimedwait.html")]
#[unsafe(no_mangle)]
pub extern "C" fn sigtimedwait(
    _set: &sigset_t,
    _info: &mut siginfo_t,
    _timeout: &timespec,
) -> c_int {
    todo!()
}

#[posix_spec("functions/sigwait.html")]
#[unsafe(no_mangle)]
pub extern "C" fn sigwait(_set: &sigset_t, _sig: &mut c_int) -> c_int {
    todo!()
}

#[posix_spec("functions/sigwaitinfo.html")]
#[unsafe(no_mangle)]
pub extern "C" fn sigwaitinfo(_set: &sigset_t, _info: &mut siginfo_t) -> c_int {
    todo!()
}

#[posix_spec("functions/str2sig.html")]
#[unsafe(no_mangle)]
pub extern "C" fn str2sig(_str: *const c_char, _sig: &mut c_int) -> c_int {
    todo!()
}
