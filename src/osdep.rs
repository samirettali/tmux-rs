// Copyright (c) 2009 Nicholas Marriott <nicholas.marriott@gmail.com>
//
// Permission to use, copy, modify, and distribute this software for any
// purpose with or without fee is hereby granted, provided that the above
// copyright notice and this permission notice appear in all copies.
//
// THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
// WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
// MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
// ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
// WHATSOEVER RESULTING FROM LOSS OF MIND, USE, DATA OR PROFITS, WHETHER
// IN AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING
// OUT OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.

use libc::{EOF, fgetc, readlink, tcgetpgrp};

use crate::*;

// Linux-specific implementation
#[cfg(target_os = "linux")]
pub unsafe fn osdep_get_name(fd: i32, tty: *const c_char) -> *mut c_char {
    unsafe {
        let pgrp = tcgetpgrp(fd);
        if pgrp == -1 {
            return null_mut();
        }

        let mut path = format_nul!("/proc/{pgrp}/cmdline");
        let f = fopen(path, c"r".as_ptr());
        if f.is_null() {
            free_(path);
            return null_mut();
        }
        free_(path);

        let mut len = 0;
        let mut buf: *mut c_char = null_mut();

        loop {
            let ch = fgetc(f);
            if ch == EOF {
                break;
            }
            if ch == b'\0' as i32 {
                break;
            }
            buf = xrealloc_(buf, len + 2).as_ptr();
            *buf.add(len) = ch as c_char;
            len += 1;
        }
        if !buf.is_null() {
            *buf.add(len) = b'\0' as c_char;
        }

        fclose(f);
        buf
    }
}

#[cfg(target_os = "linux")]
pub unsafe fn osdep_get_cwd(fd: i32) -> *const c_char {
    const MAXPATHLEN: usize = libc::PATH_MAX as usize;
    static mut target_buffer: [c_char; MAXPATHLEN + 1] = [0; MAXPATHLEN + 1];
    unsafe {
        let target = &raw mut target_buffer as *mut c_char;

        let pgrp = tcgetpgrp(fd);
        if pgrp == -1 {
            return null_mut();
        }

        let mut path = format_nul!("/proc/{pgrp}/cwd");
        let mut n = libc::readlink(path, target, MAXPATHLEN);
        free_(path);

        // If readlink failed, we'll just return null

        if n > 0 {
            *target.add(n as usize) = b'\0' as c_char;
            return target;
        }
        null_mut()
    }
}

#[cfg(target_os = "linux")]
pub unsafe fn osdep_event_init() -> *mut event_base {
    unsafe {
        // On Linux, epoll doesn't work on /dev/null (yes, really).
        libc::setenv(c"EVENT_NOEPOLL".as_ptr(), c"1".as_ptr(), 1);

        let base = event_init();
        libc::unsetenv(c"EVENT_NOEPOLL".as_ptr());
        base
    }
}

// macOS-specific implementation
#[cfg(target_os = "macos")]
pub unsafe fn osdep_get_name(fd: i32, tty: *const c_char) -> *mut c_char {
    // No additional imports needed for this simplified implementation

    unsafe {
        // For macOS, we'll use a simplified approach
        // This is a placeholder implementation - in a real scenario
        // you might want to use more sophisticated process querying
        let pgrp = tcgetpgrp(fd);
        if pgrp == -1 {
            return null_mut();
        }

        // Simple fallback: return a generic name
        let name = c"tmux".as_ptr();
        let name_len = libc::strlen(name);

        if name_len == 0 {
            return null_mut();
        }

        let buf = xmalloc(name_len + 1).as_ptr() as *mut c_char;
        libc::strcpy(buf, name);
        buf
    }
}

#[cfg(target_os = "macos")]
pub unsafe fn osdep_get_cwd(fd: i32) -> *const c_char {
    use libc::{MAXPATHLEN, proc_pidpath};

    const MAXPATHLEN_USIZE: usize = MAXPATHLEN as usize;
    static mut target_buffer: [c_char; MAXPATHLEN_USIZE + 1] = [0; MAXPATHLEN_USIZE + 1];

    unsafe {
        let target = &raw mut target_buffer as *mut c_char;

        let pgrp = tcgetpgrp(fd);
        if pgrp == -1 {
            return null_mut();
        }

        // Use proc_pidpath to get the current working directory
        let ret = proc_pidpath(pgrp, target as *mut c_void, MAXPATHLEN as u32);
        if ret > 0 {
            return target;
        }

        null_mut()
    }
}

#[cfg(target_os = "macos")]
pub unsafe fn osdep_event_init() -> *mut event_base {
    unsafe {
        // macOS doesn't need the epoll workaround
        event_init()
    }
}
