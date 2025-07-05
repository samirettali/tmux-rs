use ::libc::fprintf;

#[cfg(target_os = "linux")]
extern "C" {
    static mut stderr: *mut libc::FILE;
    fn getprogname() -> *const libc::c_char;
    fn strchr(_: *const libc::c_char, _: libc::c_int) -> *mut libc::c_char;
}

#[cfg(target_os = "macos")]
extern "C" {
    fn __stderrp() -> *mut libc::FILE;
    fn getprogname() -> *const libc::c_char;
    fn strchr(_: *const libc::c_char, _: libc::c_int) -> *mut libc::c_char;
}

#[cfg(target_os = "macos")]
unsafe fn stderr() -> *mut libc::FILE {
    unsafe { __stderrp() }
}

#[no_mangle]
pub static mut BSDopterr: libc::c_int = 1 as libc::c_int;
#[no_mangle]
pub static mut BSDoptind: libc::c_int = 1 as libc::c_int;
#[no_mangle]
pub static mut BSDoptopt: libc::c_int = 0;
#[no_mangle]
pub static mut BSDoptreset: libc::c_int = 0;
#[no_mangle]
pub static mut BSDoptarg: *mut libc::c_char = 0 as *const libc::c_char as *mut libc::c_char;
#[no_mangle]
pub unsafe fn BSDgetopt(
    mut nargc: libc::c_int,
    mut nargv: *const *mut libc::c_char,
    mut ostr: *const libc::c_char,
) -> libc::c_int {
    static mut place: *const libc::c_char = b"\0" as *const u8 as *const libc::c_char;
    let mut oli: *mut libc::c_char = 0 as *mut libc::c_char;
    if ostr.is_null() {
        return -(1 as libc::c_int);
    }
    if BSDoptreset != 0 || *place == 0 {
        BSDoptreset = 0 as libc::c_int;
        if BSDoptind >= nargc || {
            place = *nargv.offset(BSDoptind as isize);
            *place as libc::c_int != '-' as i32
        } {
            place = b"\0" as *const u8 as *const libc::c_char;
            return -(1 as libc::c_int);
        }
        if *place.offset(1 as libc::c_int as isize) as libc::c_int != 0 && {
            place = place.offset(1);
            *place as libc::c_int == '-' as i32
        } {
            if *place.offset(1 as libc::c_int as isize) != 0 {
                return '?' as i32;
            }
            BSDoptind += 1;
            BSDoptind;
            place = b"\0" as *const u8 as *const libc::c_char;
            return -(1 as libc::c_int);
        }
    }
    let fresh0 = place;
    place = place.offset(1);
    BSDoptopt = *fresh0 as libc::c_int;
    if BSDoptopt == ':' as i32 || {
        oli = strchr(ostr, BSDoptopt);
        oli.is_null()
    } {
        if BSDoptopt == '-' as i32 {
            return -(1 as libc::c_int);
        }
        if *place == 0 {
            BSDoptind += 1;
            BSDoptind;
        }
        if BSDopterr != 0 && *ostr as libc::c_int != ':' as i32 {
            #[cfg(target_os = "linux")]
            {
                fprintf(
                    stderr,
                    b"%s: unknown option -- %c\n\0" as *const u8 as *const libc::c_char,
                    getprogname(),
                    BSDoptopt,
                );
            }
            #[cfg(target_os = "macos")]
            {
                fprintf(
                    stderr(),
                    b"%s: unknown option -- %c\n\0" as *const u8 as *const libc::c_char,
                    getprogname(),
                    BSDoptopt,
                );
            }
        }
        return '?' as i32;
    }
    oli = oli.offset(1);
    if *oli as libc::c_int != ':' as i32 {
        BSDoptarg = 0 as *mut libc::c_char;
        if *place == 0 {
            BSDoptind += 1;
            BSDoptind;
        }
    } else {
        if *place != 0 {
            BSDoptarg = place as *mut libc::c_char;
        } else {
            BSDoptind += 1;
            if nargc <= BSDoptind {
                place = b"\0" as *const u8 as *const libc::c_char;
                if *ostr as libc::c_int == ':' as i32 {
                    return ':' as i32;
                }
                if BSDopterr != 0 {
                    #[cfg(target_os = "linux")]
                    {
                        fprintf(
                            stderr,
                            b"%s: option requires an argument -- %c\n\0" as *const u8
                                as *const libc::c_char,
                            getprogname(),
                            BSDoptopt,
                        );
                    }
                    #[cfg(target_os = "macos")]
                    {
                        fprintf(
                            stderr(),
                            b"%s: option requires an argument -- %c\n\0" as *const u8
                                as *const libc::c_char,
                            getprogname(),
                            BSDoptopt,
                        );
                    }
                }
                return '?' as i32;
            } else {
                BSDoptarg = *nargv.offset(BSDoptind as isize);
            }
        }
        place = b"\0" as *const u8 as *const libc::c_char;
        BSDoptind += 1;
        BSDoptind;
    }
    return BSDoptopt;
}
