use crate::unimplemented_function;
use kernel_abi::{
    SYS_PTHREAD_COND_DESTROY, SYS_PTHREAD_COND_INIT, SYS_PTHREAD_COND_SIGNAL,
    SYS_PTHREAD_COND_WAIT, SYS_PTHREAD_CONDATTR_DESTROY, SYS_PTHREAD_CONDATTR_INIT,
    SYS_PTHREAD_CONDATTR_SETCLOCK, SYS_PTHREAD_KEY_CREATE, SYS_PTHREAD_KEY_DELETE,
    SYS_PTHREAD_MUTEX_DESTROY, SYS_PTHREAD_MUTEX_INIT, SYS_PTHREAD_MUTEX_LOCK,
    SYS_PTHREAD_MUTEX_TRYLOCK, SYS_PTHREAD_MUTEX_UNLOCK, SYS_PTHREAD_MUTEXATTR_DESTROY,
    SYS_PTHREAD_MUTEXATTR_INIT, SYS_PTHREAD_MUTEXATTR_SETTYPE, SYS_PTHREAD_SETSPECIFIC,
};
use libc::{c_int, c_void};

#[unsafe(no_mangle)]
pub extern "C" fn pthread_cond_init(
    cond: *mut libc::pthread_cond_t,
    attr: *const libc::pthread_condattr_t,
) -> c_int {
    unimplemented_function(SYS_PTHREAD_COND_INIT)
}

#[unsafe(no_mangle)]
pub extern "C" fn pthread_cond_wait(
    cond: *mut libc::pthread_cond_t,
    mutex: *mut libc::pthread_mutex_t,
) {
    unimplemented_function(SYS_PTHREAD_COND_WAIT)
}

#[unsafe(no_mangle)]
pub extern "C" fn pthread_cond_signal(cond: *mut libc::pthread_cond_t) {
    unimplemented_function(SYS_PTHREAD_COND_SIGNAL)
}

#[unsafe(no_mangle)]
pub extern "C" fn pthread_cond_destroy(cond: *mut libc::pthread_cond_t) {
    unimplemented_function(SYS_PTHREAD_COND_DESTROY)
}

#[unsafe(no_mangle)]
pub extern "C" fn pthread_setspecific(key: libc::pthread_key_t, value: *const c_void) -> c_int {
    unimplemented_function(SYS_PTHREAD_SETSPECIFIC)
}

#[unsafe(no_mangle)]
pub extern "C" fn pthread_mutexattr_init(attr: *mut libc::pthread_mutexattr_t) -> c_int {
    unimplemented_function(SYS_PTHREAD_MUTEXATTR_INIT)
}

#[unsafe(no_mangle)]
pub extern "C" fn pthread_mutexattr_destroy(attr: *mut libc::pthread_mutexattr_t) -> c_int {
    unimplemented_function(SYS_PTHREAD_MUTEXATTR_DESTROY)
}

#[unsafe(no_mangle)]
pub extern "C" fn pthread_mutexattr_settype(
    attr: *mut libc::pthread_mutexattr_t,
    kind: c_int,
) -> c_int {
    unimplemented_function(SYS_PTHREAD_MUTEXATTR_SETTYPE)
}

#[unsafe(no_mangle)]
pub extern "C" fn pthread_mutex_init(
    mutex: *mut libc::pthread_mutex_t,
    attr: *const libc::pthread_mutexattr_t,
) -> c_int {
    unimplemented_function(SYS_PTHREAD_MUTEX_INIT)
}

#[unsafe(no_mangle)]
pub extern "C" fn pthread_mutex_lock(mutex: *mut libc::pthread_mutex_t) -> c_int {
    unimplemented_function(SYS_PTHREAD_MUTEX_LOCK)
}

#[unsafe(no_mangle)]
pub extern "C" fn pthread_mutex_trylock(mutex: *mut libc::pthread_mutex_t) -> c_int {
    unimplemented_function(SYS_PTHREAD_MUTEX_TRYLOCK)
}

#[unsafe(no_mangle)]
pub extern "C" fn pthread_mutex_unlock(mutex: *mut libc::pthread_mutex_t) -> c_int {
    unimplemented_function(SYS_PTHREAD_MUTEX_UNLOCK)
}

#[unsafe(no_mangle)]
pub extern "C" fn pthread_mutex_destroy(mutex: *mut libc::pthread_mutex_t) -> c_int {
    unimplemented_function(SYS_PTHREAD_MUTEX_DESTROY)
}

#[unsafe(no_mangle)]
pub extern "C" fn pthread_condattr_init(attr: *mut libc::pthread_condattr_t) -> c_int {
    unimplemented_function(SYS_PTHREAD_CONDATTR_INIT)
}

#[unsafe(no_mangle)]
pub extern "C" fn pthread_condattr_setclock(
    attr: *mut libc::pthread_condattr_t,
    clock_id: libc::clockid_t,
) -> c_int {
    unimplemented_function(SYS_PTHREAD_CONDATTR_SETCLOCK)
}

#[unsafe(no_mangle)]
pub extern "C" fn pthread_condattr_destroy(attr: *mut libc::pthread_condattr_t) -> c_int {
    unimplemented_function(SYS_PTHREAD_CONDATTR_DESTROY)
}

#[unsafe(no_mangle)]
pub extern "C" fn pthread_key_create(
    key: *mut libc::pthread_key_t,
    destructor: unsafe extern "C" fn(*mut c_void),
) -> c_int {
    unimplemented_function(SYS_PTHREAD_KEY_CREATE)
}

#[unsafe(no_mangle)]
pub extern "C" fn pthread_key_delete(key: libc::pthread_key_t) -> c_int {
    unimplemented_function(SYS_PTHREAD_KEY_DELETE)
}
