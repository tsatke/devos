use crate::locale::locale_t;
use crate::signal::sigevent;
use crate::sys_types::{clock_t, clockid_t, pid_t, size_t, time_t, timer_t};
use core::ffi::{c_char, c_double, c_int, c_long};
use muffin_libc_spec_comment::posix_spec;

#[repr(C)]
pub struct tm {
    pub tm_sec: c_int,
    pub tm_min: c_int,
    pub tm_hour: c_int,
    pub tm_mday: c_int,
    pub tm_mon: c_int,
    pub tm_year: c_int,
    pub tm_wday: c_int,
    pub tm_yday: c_int,
    pub tm_isdst: c_int,
    pub tm_gmtoff: c_long,
    pub tm_zone: *const c_char,
}

#[repr(C)]
pub struct timespec {
    pub tv_sec: time_t,
    pub tv_nsec: c_long,
}

#[repr(C)]
pub struct itimerspec {
    pub it_interval: timespec,
    pub it_value: timespec,
}

#[posix_spec("functions/clock.html")]
#[unsafe(no_mangle)]
pub extern "C" fn clock() -> clock_t {
    todo!()
}

#[posix_spec("functions/clock_getcpuclockid.html")]
#[unsafe(no_mangle)]
pub extern "C" fn clock_getcpuclockid(_pid: pid_t, _clock_id: &clockid_t) -> c_int {
    todo!()
}

#[posix_spec("functions/clock_getres.html")]
#[unsafe(no_mangle)]
pub extern "C" fn clock_getres(_clock_id: clockid_t, _ts: &mut timespec) -> c_int {
    todo!()
}

#[posix_spec("functions/clock_gettime.html")]
#[unsafe(no_mangle)]
pub extern "C" fn clock_gettime(_clock_id: clockid_t, _ts: &mut timespec) -> c_int {
    todo!()
}

#[posix_spec("functions/clock_nanosleep.html")]
#[unsafe(no_mangle)]
pub extern "C" fn clock_nanosleep(
    _clock_id: clockid_t,
    _flags: c_int,
    _request: &timespec,
    _remaining: &mut timespec,
) -> c_int {
    todo!()
}

#[posix_spec("functions/clock_settime.html")]
#[unsafe(no_mangle)]
pub extern "C" fn clock_settime(_clock_id: clockid_t, _ts: &timespec) -> c_int {
    todo!()
}

#[posix_spec("functions/ctime.html")]
#[unsafe(no_mangle)]
pub extern "C" fn ctime(_time: &time_t) -> *const c_char {
    todo!()
}

#[posix_spec("functions/difftime.html")]
#[unsafe(no_mangle)]
pub extern "C" fn difftime(time0: time_t, time1: time_t) -> c_double {
    (time1 - time0) as c_double
}

#[posix_spec("functions/getdate.html")]
#[unsafe(no_mangle)]
pub extern "C" fn getdate(_date: *const c_char) -> *mut tm {
    todo!()
}

#[posix_spec("functions/gmtime.html")]
#[unsafe(no_mangle)]
pub extern "C" fn gmtime(_time: &time_t) -> *mut tm {
    todo!()
}

#[posix_spec("functions/gmtime_r.html")]
#[unsafe(no_mangle)]
pub extern "C" fn gmtime_r(_time: &time_t, _result: &mut tm) -> *mut tm {
    todo!()
}

#[posix_spec("functions/localtime.html")]
#[unsafe(no_mangle)]
pub extern "C" fn localtime(_time: &time_t) -> *mut tm {
    todo!()
}

#[posix_spec("functions/localtime_r.html")]
#[unsafe(no_mangle)]
pub extern "C" fn localtime_r(_time: &time_t, _result: &mut tm) -> *mut tm {
    todo!()
}

#[posix_spec("functions/mktime.html")]
#[unsafe(no_mangle)]
pub extern "C" fn mktime(_tm: &mut tm) -> time_t {
    todo!()
}

#[posix_spec("functions/nanosleep.html")]
#[unsafe(no_mangle)]
pub extern "C" fn nanosleep(_req: &timespec, _rem: &mut timespec) -> c_int {
    todo!()
}

#[posix_spec("functions/strftime.html")]
#[unsafe(no_mangle)]
pub extern "C" fn strftime(
    _s: *mut c_char,
    _maxsize: size_t,
    _format: *const c_char,
    _tm: &tm,
) -> c_int {
    todo!()
}

#[posix_spec("functions/strftime_l.html")]
#[unsafe(no_mangle)]
pub extern "C" fn strftime_l(
    _s: *mut c_char,
    _maxsize: size_t,
    _format: *const c_char,
    _tm: &tm,
    _locale: locale_t,
) -> c_int {
    todo!()
}

#[posix_spec("functions/strptime.html")]
#[unsafe(no_mangle)]
pub extern "C" fn strptime(
    _buf: *const c_char,
    _format: *const c_char,
    _tm: &mut tm,
) -> *mut c_char {
    todo!()
}

#[posix_spec("functions/time.html")]
#[unsafe(no_mangle)]
pub extern "C" fn time(_tloc: &mut time_t) -> time_t {
    todo!()
}

#[posix_spec("functions/timer_create.html")]
#[unsafe(no_mangle)]
pub extern "C" fn timer_create(
    _clock_id: clockid_t,
    _evp: &mut sigevent,
    _timerid: &mut timer_t,
) -> c_int {
    todo!()
}

#[posix_spec("functions/timer_delete.html")]
#[unsafe(no_mangle)]
pub extern "C" fn timer_delete(_timerid: timer_t) -> c_int {
    todo!()
}

#[posix_spec("functions/timer_getoverrun.html")]
#[unsafe(no_mangle)]
pub extern "C" fn timer_getoverrun(_timerid: timer_t) -> c_int {
    todo!()
}

#[posix_spec("functions/timer_gettime.html")]
#[unsafe(no_mangle)]
pub extern "C" fn timer_gettime(_timerid: timer_t, _its: &mut itimerspec) -> c_int {
    todo!()
}

#[posix_spec("functions/timer_settime.html")]
#[unsafe(no_mangle)]
pub extern "C" fn timer_settime(
    _timerid: timer_t,
    _flags: c_int,
    _new_value: &itimerspec,
    _old_value: &mut itimerspec,
) -> c_int {
    todo!()
}

#[posix_spec("functions/timespec_get.html")]
#[unsafe(no_mangle)]
pub extern "C" fn timespec_get(_ts: &mut timespec, _base: c_int) -> c_int {
    todo!()
}

#[posix_spec("functions/tzset.html")]
#[unsafe(no_mangle)]
pub extern "C" fn tzset() {
    todo!()
}
