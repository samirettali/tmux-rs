use core::ffi::{c_char, c_int, c_void};

pub mod b64;
pub mod fdforkpty;
pub mod getdtablecount;
pub mod getprogname;
pub mod imsg;
pub mod imsg_buffer;
pub mod queue;
pub mod systemd;
pub mod tree;

mod closefrom;
mod fgetln;
mod freezero;
mod getpeereid;
mod recallocarray;
mod setproctitle;
mod strlcat;
mod strlcpy;
mod strtonum;
mod unvis;
mod vis;

pub use closefrom::closefrom;
pub use fgetln::fgetln;
pub use freezero::freezero;
pub use getpeereid::getpeereid;
pub use recallocarray::recallocarray;
pub use setproctitle::setproctitle_;
pub use strlcat::strlcat;
pub use strlcpy::strlcpy;
pub use strtonum::strtonum;
pub use systemd::systemd_create_socket;
pub use unvis::strunvis;
pub use vis::*;

pub(crate) use queue::{TAILQ_HEAD_INITIALIZER, impl_tailq_entry, tailq_insert_head};
pub(crate) use tree::RB_GENERATE;

#[rustfmt::skip]
unsafe extern "C" {
    pub static mut optreset: c_int;
    pub static mut optarg: *mut c_char;
    pub static mut optind: c_int;
    pub fn getopt(___argc: c_int, ___argv: *const *mut c_char, __shortopts: *const c_char) -> c_int;
    pub fn bsd_getopt(argc: c_int, argv: *const *mut c_char, shortopts: *const c_char) -> c_int;
}

pub const HOST_NAME_MAX: usize = 255;

pub const WAIT_ANY: libc::pid_t = -1;

pub const ACCESSPERMS: libc::mode_t = libc::S_IRWXU | libc::S_IRWXG | libc::S_IRWXO;

// #define S_ISDIR(mode)  (((mode) & S_IFMT) == S_IFDIR)
// TODO move this to a better spot
#[allow(non_snake_case)]
#[inline]
pub fn S_ISDIR(mode: u32) -> bool {
    mode & (libc::S_IFMT as u32) == (libc::S_IFDIR as u32)
}

// macOS compatibility functions
#[cfg(target_os = "macos")]
pub unsafe fn explicit_bzero(ptr: *mut c_void, size: usize) {
    // Use memset_s if available, otherwise fall back to memset + compiler barrier
    libc::memset(ptr, 0, size);
    std::sync::atomic::compiler_fence(std::sync::atomic::Ordering::SeqCst);
}

#[cfg(target_os = "macos")]
pub unsafe fn reallocarray(ptr: *mut c_void, nmemb: usize, size: usize) -> *mut c_void {
    // Check for overflow
    if nmemb > 0 && size > usize::MAX / nmemb {
        crate::errno!(libc::ENOMEM);
        return std::ptr::null_mut();
    }

    let total_size = nmemb * size;
    if total_size == 0 {
        return std::ptr::null_mut();
    }

    libc::realloc(ptr, total_size)
}

#[cfg(target_os = "macos")]
pub unsafe fn posix_basename(path: *const c_char) -> *mut c_char {
    // Simple implementation - just return the part after the last '/'
    let mut last_slash = path;
    let mut current = path;

    while *current != 0 {
        if *current == b'/' as c_char {
            last_slash = current.add(1);
        }
        current = current.add(1);
    }

    if *last_slash == 0 {
        return c".".as_ptr() as *mut c_char;
    }

    last_slash as *mut c_char
}

#[cfg(target_os = "macos")]
pub unsafe fn prctl(_option: c_int, _arg2: *const c_char) -> c_int {
    // macOS doesn't have prctl, so this is a no-op
    0
}

// Define missing constants for macOS
#[cfg(target_os = "macos")]
pub const PR_SET_NAME: c_int = 15;
#[cfg(target_os = "macos")]
pub const _SC_MB_LEN_MAX: c_int = 4;

// extern crate compat_derive;
// pub use compat_derive::TailQEntry;
