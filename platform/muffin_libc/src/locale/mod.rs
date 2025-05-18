use core::ffi::{c_char, c_int};
use muffin_libc_spec_comment::posix_spec;

#[repr(C)]
pub struct lconv {
    pub currency_symbol: *const c_char,
    pub decimal_point: *const c_char,
    pub frac_digits: c_char,
    pub grouping: *const c_char,
    pub int_curr_symbol: *const c_char,
    pub int_frac_digits: c_char,
    pub int_n_cs_precedes: c_char,
    pub int_n_sep_by_space: c_char,
    pub int_n_sign_posn: c_char,
    pub int_p_cs_precedes: c_char,
    pub int_p_sep_by_space: c_char,
    pub int_p_sign_posn: c_char,
    pub mon_decimal_point: *const c_char,
    pub mon_grouping: *const c_char,
    pub mon_thousands_sep: *const c_char,
    pub negative_sign: *const c_char,
    pub n_cs_precedes: c_char,
    pub n_sep_by_space: c_char,
    pub n_sign_posn: c_char,
    pub positive_sign: *const c_char,
    pub p_cs_precedes: c_char,
    pub p_sep_by_space: c_char,
    pub p_sign_posn: c_char,
    pub thousands_sep: *const c_char,
}

#[repr(C)]
pub struct locale_t {
    pub _locale: *const c_char,
}

#[posix_spec("functions/duplocale.html")]
#[unsafe(no_mangle)]
pub extern "C" fn duplocale(_locale: locale_t) -> locale_t {
    todo!()
}

#[posix_spec("functions/freelocale.html")]
#[unsafe(no_mangle)]
pub extern "C" fn freelocale(_locale: locale_t) {
    todo!()
}

#[posix_spec("functions/getlocalename_l.html")]
#[unsafe(no_mangle)]
pub extern "C" fn getlocalename_l(_category: c_int, _locale: locale_t) -> *const c_char {
    todo!()
}

#[posix_spec("functions/localeconv.html")]
#[unsafe(no_mangle)]
pub extern "C" fn localeconv() -> *const lconv {
    todo!()
}

#[posix_spec("functions/newlocale.html")]
#[unsafe(no_mangle)]
pub extern "C" fn newlocale(
    _category_mask: c_int,
    _locale: *const c_char,
    _base_locale: locale_t,
) -> locale_t {
    todo!()
}

#[posix_spec("functions/setlocale.html")]
#[unsafe(no_mangle)]
pub extern "C" fn setlocale(_category: c_int, _locale: *const c_char) -> *const c_char {
    todo!()
}

// locale_t      uselocale (locale_t);
#[posix_spec("functions/uselocale.html")]
#[unsafe(no_mangle)]
pub extern "C" fn uselocale(_locale: locale_t) -> locale_t {
    todo!()
}
