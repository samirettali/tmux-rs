use core::ffi::{c_char, c_void};

use libc::timeval;

pub type wchar_t = core::ffi::c_int;

#[inline]
pub unsafe fn bsearch_<T>(
    key: *const T,
    base: *const T,
    num: usize,
    size: usize,
    compar: unsafe extern "C" fn(*const c_void, *const c_void) -> i32,
) -> *mut T {
    unsafe { ::libc::bsearch(key.cast(), base.cast(), num, size, Some(compar)).cast() }
}

#[inline]
pub unsafe fn bsearch__<T>(
    key: *const T,
    base: *const T,
    num: usize,
    compar: unsafe extern "C" fn(*const c_void, *const c_void) -> i32,
) -> *mut T {
    unsafe { ::libc::bsearch(key.cast(), base.cast(), num, size_of::<T>(), Some(compar)).cast() }
}

#[cfg(target_os = "linux")]
#[macro_export]
macro_rules! errno {
    () => {
        unsafe { *::libc::__errno_location() }
    };
    ($value:expr) => {
        unsafe {
            *::libc::__errno_location() = $value;
        }
    };
}

#[cfg(target_os = "macos")]
#[macro_export]
macro_rules! errno {
    () => {
        unsafe { *::libc::__error() }
    };
    ($value:expr) => {
        unsafe {
            *::libc::__error() = $value;
        }
    };
}
pub(crate) use errno;

#[allow(non_snake_case)]
#[inline]
pub fn MB_CUR_MAX() -> usize {
    unsafe extern "C" {
        unsafe fn __ctype_get_mb_cur_max() -> usize;
    }
    unsafe { __ctype_get_mb_cur_max() }
}

unsafe extern "C" {
    pub fn wcwidth(c: wchar_t) -> i32;
    pub fn mbtowc(pwc: *mut wchar_t, s: *const c_char, n: usize) -> i32;
    pub fn wctomb(s: *mut c_char, wc: wchar_t) -> i32;
}

#[inline]
pub unsafe fn memset0<T>(dest: *mut T) -> *mut T {
    unsafe { libc::memset(dest.cast(), 0, size_of::<T>()).cast() }
}

#[inline]
pub unsafe fn timerclear(tv: *mut timeval) {
    // implemented as a macro by most libc's
    unsafe {
        (*tv).tv_sec = 0;
        (*tv).tv_usec = 0;
    }
}

/// result must be initialized after this function
#[inline]
pub unsafe fn timersub(a: *const timeval, b: *const timeval, result: *mut timeval) {
    // implemented as a macro by most libc's
    unsafe {
        (*result).tv_sec = (*a).tv_sec - (*b).tv_sec;
        (*result).tv_usec = (*a).tv_usec - (*b).tv_usec;
        if (*result).tv_usec < 0 {
            (*result).tv_sec -= 1;
            (*result).tv_usec += 1000000;
        }
    }
}

pub struct timer(*const libc::timeval);
impl timer {
    pub unsafe fn new(ptr: *const libc::timeval) -> Self {
        Self(ptr)
    }
}
impl Eq for timer {}
impl PartialEq for timer {
    fn eq(&self, other: &Self) -> bool {
        unsafe { (*self.0).tv_sec == (*other.0).tv_sec && (*self.0).tv_usec == (*other.0).tv_usec }
    }
}
impl Ord for timer {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        unsafe {
            if (*self.0).tv_sec == (*other.0).tv_sec {
                (*self.0).tv_usec.cmp(&(*other.0).tv_usec)
            } else {
                (*self.0).tv_sec.cmp(&(*other.0).tv_sec)
            }
        }
    }
}
impl PartialOrd for timer {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

pub unsafe fn strcmp_(left: *const c_char, right: &'static str) -> std::cmp::Ordering {
    unsafe {
        for (i, r_ch) in right.bytes().enumerate() {
            let l_ch = *left.add(i) as u8;

            if l_ch == b'\0' {
                return std::cmp::Ordering::Less;
            }

            match l_ch.cmp(&r_ch) {
                std::cmp::Ordering::Equal => continue,
                value => return value,
            }
        }

        if *left.add(right.len()) == b'\0' as i8 {
            std::cmp::Ordering::Equal
        } else {
            std::cmp::Ordering::Greater
        }
    }
}

pub unsafe fn strcasecmp_(left: *const c_char, right: &'static str) -> std::cmp::Ordering {
    unsafe {
        for (i, r_ch) in right.bytes().enumerate() {
            let l_ch = *left.add(i) as u8;

            if l_ch == b'\0' {
                return std::cmp::Ordering::Less;
            }

            match l_ch.to_ascii_lowercase().cmp(&r_ch.to_ascii_lowercase()) {
                std::cmp::Ordering::Equal => continue,
                value => return value,
            }
        }

        if *left.add(right.len()) == b'\0' as i8 {
            std::cmp::Ordering::Equal
        } else {
            std::cmp::Ordering::Greater
        }
    }
}

pub unsafe fn streq_(left: *const c_char, right: &'static str) -> bool {
    unsafe { matches!(strcmp_(left, right), std::cmp::Ordering::Equal) }
}

pub unsafe fn strcaseeq_(left: *const c_char, right: &'static str) -> bool {
    unsafe { matches!(strcasecmp_(left, right), std::cmp::Ordering::Equal) }
}
