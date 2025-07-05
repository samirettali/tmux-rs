use std::ffi::CStr;
use std::ptr;

#[cfg(target_os = "macos")]
unsafe extern "C" {
    fn _NSGetProgname() -> *const libc::c_char;
}

#[cfg(target_os = "linux")]
unsafe extern "C" {
    static mut program_invocation_short_name: *mut libc::c_char;
}

pub unsafe fn getprogname() -> *const libc::c_char {
    #[cfg(target_os = "macos")]
    {
        unsafe { _NSGetProgname() }
    }

    #[cfg(target_os = "linux")]
    {
        unsafe { program_invocation_short_name }
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        // Fallback for other platforms
        c"tmux".as_ptr()
    }
}
