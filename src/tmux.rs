// Copyright (c) 2007 Nicholas Marriott <nicholas.marriott@gmail.com>
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
use crate::xmalloc::xstrndup;
use crate::*;

unsafe extern "C" {
    // TODO move/remove
    fn errx(_: c_int, _: *const c_char, ...);
    fn err(_: c_int, _: *const c_char, ...);

    fn tzset();
}

use crate::compat::{S_ISDIR, fdforkpty::getptmfd, getprogname::getprogname, optarg, optind};
use libc::{
    CLOCK_MONOTONIC, CLOCK_REALTIME, CODESET, EEXIST, F_GETFL, F_SETFL, LC_CTYPE, LC_TIME,
    O_NONBLOCK, PATH_MAX, S_IRWXO, S_IRWXU, X_OK, access, clock_gettime, fcntl, getcwd, getenv,
    getopt, getpwuid, getuid, lstat, mkdir, nl_langinfo, printf, realpath, setlocale, stat,
    strcasecmp, strcasestr, strchr, strcspn, strerror, strncmp, strrchr, strstr, timespec,
};

pub static mut global_options: *mut options = null_mut();

pub static mut global_s_options: *mut options = null_mut();

pub static mut global_w_options: *mut options = null_mut();

pub static mut global_environ: *mut environ = null_mut();

pub static mut start_time: timeval = unsafe { zeroed() };

pub static mut socket_path: *const c_char = null_mut();

pub static mut ptm_fd: c_int = -1;

pub static mut shell_command: *mut c_char = null_mut();

pub fn usage() -> ! {
    unsafe {
        libc::fprintf(get_stderr(), c"usage: %s [-2CDlNuVv] [-c shell-command] [-f file] [-L socket-name]\n            [-S socket-path] [-T features] [command [flags]]\n".as_ptr(), getprogname());
        std::process::exit(1)
    }
}

pub unsafe fn getshell() -> *const c_char {
    unsafe {
        let shell = getenv(c"SHELL".as_ptr());
        if checkshell(shell) {
            return shell;
        }

        let pw = getpwuid(getuid());
        if !pw.is_null() && checkshell((*pw).pw_shell) {
            return (*pw).pw_shell;
        }

        _PATH_BSHELL
    }
}

pub unsafe fn checkshell(shell: *const c_char) -> bool {
    unsafe {
        if shell.is_null() || *shell != b'/' as c_char {
            return false;
        }
        if areshell(shell) != 0 {
            return false;
        }
        if access(shell, X_OK) != 0 {
            return false;
        }
    }
    true
}

pub unsafe fn areshell(shell: *const c_char) -> c_int {
    unsafe {
        let ptr = strrchr(shell, b'/' as c_int);
        let ptr = if !ptr.is_null() {
            ptr.wrapping_add(1)
        } else {
            shell
        };
        let mut progname = getprogname();
        if *progname == b'-' as c_char {
            progname = progname.wrapping_add(1);
        }
        if libc::strcmp(ptr, progname) == 0 {
            1
        } else {
            0
        }
    }
}

pub unsafe fn expand_path(path: *const c_char, home: *const c_char) -> *mut c_char {
    unsafe {
        let mut expanded: *mut c_char = null_mut();
        let mut end: *const c_char = null_mut();

        if strncmp(path, c"~/".as_ptr(), 2) == 0 {
            if home.is_null() {
                return null_mut();
            }
            return format_nul!("{}{}", _s(home), _s(path.add(1)));
        }

        if *path == b'$' as c_char {
            end = strchr(path, b'/' as i32);
            let name = if end.is_null() {
                xstrdup(path.add(1)).cast().as_ptr()
            } else {
                xstrndup(path.add(1), end.addr() - path.addr() - 1)
                    .cast()
                    .as_ptr()
            };
            let value = environ_find(global_environ, name);
            free_(name);
            if value.is_null() {
                return null_mut();
            }
            if end.is_null() {
                end = c"".as_ptr();
            }
            return format_nul!("{}{}", _s(transmute_ptr((*value).value)), _s(end));
        }

        xstrdup(path).cast().as_ptr()
    }
}

unsafe fn expand_paths(
    s: *const c_char,
    paths: *mut *mut *mut c_char,
    n: *mut u32,
    ignore_errors: i32,
) {
    unsafe {
        let home = find_home();
        let mut next: *const c_char = null_mut();
        let mut resolved: [c_char; PATH_MAX as usize] = zeroed(); // TODO use unint version
        let mut path = null_mut();

        let func = "expand_paths";

        *paths = null_mut();
        *n = 0;

        let mut tmp: *mut c_char = xstrdup(s).cast().as_ptr();
        let copy = tmp;
        while {
            next = strsep(&raw mut tmp as _, c":".as_ptr().cast());
            !next.is_null()
        } {
            let expanded = expand_path(next, home);
            if expanded.is_null() {
                log_debug!("{}: invalid path: {}", func, _s(next));
                continue;
            }
            if realpath(expanded, resolved.as_mut_ptr()).is_null() {
                log_debug!(
                    "{}: realpath(\"{}\") failed: {}",
                    func,
                    _s(expanded),
                    _s(strerror(errno!())),
                );
                if ignore_errors != 0 {
                    free_(expanded);
                    continue;
                }
                path = expanded;
            } else {
                path = xstrdup(resolved.as_ptr()).cast().as_ptr();
                free_(expanded);
            }
            let mut i = 0;
            for j in 0..*n {
                i = j;
                if libc::strcmp(path as _, *(*paths).add(i as usize)) == 0 {
                    break;
                }
            }
            if i != *n {
                log_debug!("{}: duplicate path: {}", func, _s(path));
                free_(path);
                continue;
            }
            *paths = xreallocarray_::<*mut c_char>(*paths, (*n + 1) as usize).as_ptr();
            *(*paths).add((*n) as usize) = path;
            *n += 1;
        }
        free_(copy);
    }
}

unsafe fn make_label(mut label: *const c_char, cause: *mut *mut c_char) -> *const c_char {
    let mut paths: *mut *mut c_char = null_mut();
    let mut path: *mut c_char = null_mut();
    let mut base: *mut c_char = null_mut();
    let mut sb: stat = unsafe { zeroed() }; // TODO use uninit
    let mut n: u32 = 0;

    unsafe {
        'fail: {
            *cause = null_mut();
            if label.is_null() {
                label = c"default".as_ptr();
            }
            let uid = getuid();

            expand_paths(TMUX_SOCK.as_ptr(), &raw mut paths, &raw mut n, 1);
            if n == 0 {
                *cause = format_nul!("no suitable socket path");
                return null_mut();
            }
            path = *paths; /* can only have one socket! */
            for i in 1..n {
                free_(*paths.add(i as usize));
            }
            free_(paths);

            base = format_nul!("{}/tmux-{}", _s(path), uid);
            free_(path);
            if mkdir(base, S_IRWXU) != 0 && errno!() != EEXIST {
                *cause = format_nul!(
                    "couldn't create directory {} ({})",
                    _s(base),
                    _s(strerror(errno!()))
                );
                break 'fail;
            }
            if lstat(base, &raw mut sb) != 0 {
                *cause = format_nul!(
                    "couldn't read directory {} ({})",
                    _s(base),
                    _s(strerror(errno!())),
                );
                break 'fail;
            }
            if !S_ISDIR(sb.st_mode.into()) {
                *cause = format_nul!("{} is not a directory", _s(base));
                break 'fail;
            }
            if sb.st_uid != uid || (sb.st_mode & S_IRWXO) != 0 {
                *cause = format_nul!("directory {} has unsafe permissions", _s(base));
                break 'fail;
            }
            path = format_nul!("{}/{}", _s(base), _s(label));
            free_(base);
            return path;
        }

        // fail:
        free_(base);
        null_mut()
    }
}

pub unsafe fn shell_argv0(shell: *const c_char, is_login: c_int) -> *mut c_char {
    unsafe {
        let slash = strrchr(shell, b'/' as _);
        let name = if !slash.is_null() && *slash.add(1) != b'\0' as c_char {
            slash.add(1)
        } else {
            shell
        };

        if is_login != 0 {
            format_nul!("-{}", _s(name))
        } else {
            format_nul!("{}", _s(name))
        }
    }
}

pub unsafe fn setblocking(fd: c_int, state: c_int) {
    unsafe {
        let mut mode = fcntl(fd, F_GETFL);

        if mode != -1 {
            if state == 0 {
                mode |= O_NONBLOCK;
            } else {
                mode &= !O_NONBLOCK;
            }
            fcntl(fd, F_SETFL, mode);
        }
    }
}

pub unsafe fn get_timer() -> u64 {
    unsafe {
        let mut ts: timespec = zeroed();
        //We want a timestamp in milliseconds suitable for time measurement,
        //so prefer the monotonic clock.
        if clock_gettime(CLOCK_MONOTONIC, &raw mut ts) != 0 {
            clock_gettime(CLOCK_REALTIME, &raw mut ts);
        }
        (ts.tv_sec as u64 * 1000) + (ts.tv_nsec as u64 / 1000000)
    }
}

pub unsafe fn find_cwd() -> *mut c_char {
    static mut cwd: [c_char; PATH_MAX as usize] = [0; PATH_MAX as usize];
    unsafe {
        let mut resolved1: [c_char; PATH_MAX as usize] = [0; PATH_MAX as usize];
        let mut resolved2: [c_char; PATH_MAX as usize] = [0; PATH_MAX as usize];

        if getcwd(&raw mut cwd as _, size_of::<[c_char; PATH_MAX as usize]>()).is_null() {
            return null_mut();
        }
        let pwd = getenv(c"PWD".as_ptr());
        if pwd.is_null() || *pwd == b'\0' as c_char {
            return &raw mut cwd as _;
        }

        //We want to use PWD so that symbolic links are maintained,
        //but only if it matches the actual working directory.

        if realpath(pwd, &raw mut resolved1 as _).is_null() {
            return &raw mut cwd as _;
        }
        if realpath(&raw mut cwd as _, &raw mut resolved2 as _).is_null() {
            return &raw mut cwd as _;
        }
        if libc::strcmp(&raw mut resolved1 as _, &raw mut resolved2 as _) != 0 {
            return &raw mut cwd as _;
        }
        pwd
    }
}

pub unsafe fn find_home() -> *mut c_char {
    static mut home: *mut c_char = null_mut();

    unsafe {
        if !home.is_null() {
            home
        } else {
            home = getenv(c"HOME".as_ptr());
            if home.is_null() || *home == b'\0' as c_char {
                let pw = getpwuid(getuid());
                if !pw.is_null() {
                    home = (*pw).pw_dir;
                } else {
                    home = null_mut();
                }
            }

            home
        }
    }
}

pub fn getversion() -> &'static str {
    "3.5rs"
}

pub fn getversion_c() -> *const c_char {
    c"3.5rs".as_ptr()
}

/// entrypoint for tmux binary
#[cfg_attr(not(test), unsafe(no_mangle))]
pub unsafe fn main(mut argc: i32, mut argv: *mut *mut c_char, env: *mut *mut c_char) {
    std::panic::set_hook(Box::new(|panic_info| {
        let backtrace = std::backtrace::Backtrace::capture();
        let err_str = format!("{backtrace:#?}");
        std::fs::write("client-panic.txt", err_str).unwrap();
    }));

    unsafe {
        // setproctitle_init(argc, argv.cast(), env.cast());
        let mut cause: *mut c_char = null_mut();
        let mut path: *const c_char = null_mut();
        let mut label: *mut c_char = null_mut();
        let mut feat: i32 = 0;
        let mut fflag: i32 = 0;
        let mut flags: client_flag = client_flag::empty();

        if setlocale(LC_CTYPE, c"en_US.UTF-8".as_ptr()).is_null()
            && setlocale(LC_CTYPE, c"C.UTF-8".as_ptr()).is_null()
        {
            if setlocale(LC_CTYPE, c"".as_ptr()).is_null() {
                errx(1, c"invalid LC_ALL, LC_CTYPE or LANG".as_ptr());
            }
            let s = nl_langinfo(CODESET);
            if strcasecmp(s, c"UTF-8".as_ptr()) != 0 && strcasecmp(s, c"UTF8".as_ptr()) != 0 {
                errx(1, c"need UTF-8 locale (LC_CTYPE) but have %s".as_ptr(), s);
            }
        }

        setlocale(LC_TIME, c"".as_ptr());
        tzset();

        if **argv == b'-' as c_char {
            flags = client_flag::LOGIN;
        }

        global_environ = environ_create().as_ptr();

        let mut var = environ;
        while !(*var).is_null() {
            environ_put(global_environ, *var, 0);
            var = var.add(1);
        }

        let cwd = find_cwd();
        if !cwd.is_null() {
            environ_set!(global_environ, c"PWD".as_ptr(), 0, "{}", _s(cwd));
        }
        expand_paths(
            TMUX_CONF.as_ptr(),
            &raw mut cfg_files,
            &raw mut cfg_nfiles,
            1,
        );

        let mut opt;
        while {
            opt = getopt(argc, argv, c"2c:CDdf:lL:NqS:T:uUvV".as_ptr());
            opt != -1
        } {
            match opt as u8 {
                b'2' => tty_add_features(&raw mut feat, c"256".as_ptr(), c":,".as_ptr()),
                b'c' => shell_command = optarg,
                b'D' => flags |= client_flag::NOFORK,
                b'C' => {
                    if flags.intersects(client_flag::CONTROL) {
                        flags |= client_flag::CONTROLCONTROL;
                    } else {
                        flags |= client_flag::CONTROL;
                    }
                }
                b'f' => {
                    if fflag == 0 {
                        fflag = 1;
                        for i in 0..cfg_nfiles {
                            free((*cfg_files.add(i as usize)) as _);
                        }
                        cfg_nfiles = 0;
                    }
                    cfg_files =
                        xreallocarray_::<*mut c_char>(cfg_files, cfg_nfiles as usize + 1).as_ptr();
                    *cfg_files.add(cfg_nfiles as usize) = xstrdup(optarg).cast().as_ptr();
                    cfg_nfiles += 1;
                    cfg_quiet = 0;
                }
                b'V' => {
                    println!("tmux {}", getversion());
                    std::process::exit(0);
                }
                b'l' => flags |= client_flag::LOGIN,
                b'L' => {
                    free(label as _);
                    label = xstrdup(optarg).cast().as_ptr();
                }
                b'N' => flags |= client_flag::NOSTARTSERVER,
                b'q' => (),
                b'S' => {
                    free(path as _);
                    path = xstrdup(optarg).cast().as_ptr();
                }
                b'T' => tty_add_features(&raw mut feat, optarg, c":,".as_ptr()),
                b'u' => flags |= client_flag::UTF8,
                b'v' => log_add_level(),
                _ => usage(),
            }
        }
        argc -= optind;
        argv = argv.add(optind as usize);

        if !shell_command.is_null() && argc != 0 {
            usage();
        }
        if flags.intersects(client_flag::NOFORK) && argc != 0 {
            usage();
        }

        ptm_fd = getptmfd();
        if ptm_fd == -1 {
            err(1, c"getptmfd".as_ptr());
        }

        /*
        // TODO no pledge on linux
            if pledge("stdio rpath wpath cpath flock fattr unix getpw sendfd recvfd proc exec tty ps", null_mut()) != 0 {
                err(1, "pledge");
        }
        */

        // tmux is a UTF-8 terminal, so if TMUX is set, assume UTF-8.
        // Otherwise, if the user has set LC_ALL, LC_CTYPE or LANG to contain
        // UTF-8, it is a safe assumption that either they are using a UTF-8
        // terminal, or if not they know that output from UTF-8-capable
        // programs may be wrong.
        if !getenv(c"TMUX".as_ptr()).is_null() {
            flags |= client_flag::UTF8;
        } else {
            let mut s = getenv(c"LC_ALL".as_ptr()) as *const c_char;
            if s.is_null() || *s == b'\0' as c_char {
                s = getenv(c"LC_CTYPE".as_ptr()) as *const c_char;
            }
            if s.is_null() || *s == b'\0' as c_char {
                s = getenv(c"LANG".as_ptr()) as *const c_char;
            }
            if s.is_null() || *s == b'\0' as c_char {
                s = c"".as_ptr();
            }
            if !strcasestr(s, c"UTF-8".as_ptr()).is_null()
                || !strcasestr(s, c"UTF8".as_ptr()).is_null()
            {
                flags |= client_flag::UTF8;
            }
        }

        global_options = options_create(null_mut());
        global_s_options = options_create(null_mut());
        global_w_options = options_create(null_mut());

        let mut oe: *const options_table_entry = &raw const options_table as _;
        while !(*oe).name.is_null() {
            if (*oe).scope & OPTIONS_TABLE_SERVER != 0 {
                options_default(global_options, oe);
            }
            if (*oe).scope & OPTIONS_TABLE_SESSION != 0 {
                options_default(global_s_options, oe);
            }
            if (*oe).scope & OPTIONS_TABLE_WINDOW != 0 {
                options_default(global_w_options, oe);
            }
            oe = oe.add(1);
        }

        // The default shell comes from SHELL or from the user's passwd entry if available.
        options_set_string!(
            global_s_options,
            c"default-shell".as_ptr(),
            0,
            "{}",
            _s(getshell()),
        );

        // Override keys to vi if VISUAL or EDITOR are set.
        let mut s = getenv(c"VISUAL".as_ptr());
        if !s.is_null()
            || ({
                s = getenv(c"EDITOR".as_ptr());
                !s.is_null()
            })
        {
            options_set_string!(global_options, c"editor".as_ptr(), 0, "{}", _s(s));
            if !strrchr(s, b'/' as _).is_null() {
                s = strrchr(s, b'/' as _).add(1);
            }
            let keys = if !strstr(s, c"vi".as_ptr()).is_null() {
                modekey::MODEKEY_VI
            } else {
                modekey::MODEKEY_EMACS
            };
            options_set_number(global_s_options, c"status-keys".as_ptr(), keys as _);
            options_set_number(global_w_options, c"mode-keys".as_ptr(), keys as _);
        }

        // If socket is specified on the command-line with -S or -L, it is
        // used. Otherwise, $TMUX is checked and if that fails "default" is
        // used.
        if path.is_null() && label.is_null() {
            s = getenv(c"TMUX".as_ptr());
            if !s.is_null() && *s != b'\0' as c_char && *s != b',' as c_char {
                let tmp: *mut c_char = xstrdup(s).cast().as_ptr();
                *tmp.add(strcspn(tmp, c",".as_ptr())) = b'\0' as c_char;
                path = tmp;
            }
        }
        if path.is_null() {
            path = make_label(label.cast(), &raw mut cause);
            if path.is_null() {
                if !cause.is_null() {
                    libc::fprintf(get_stderr(), c"%s\n".as_ptr(), cause);
                    free(cause as _);
                }
                std::process::exit(1);
            }
            flags |= client_flag::DEFAULTSOCKET;
        }
        socket_path = path;
        free_(label);

        // Pass control to the client.
        std::process::exit(client_main(osdep_event_init(), argc, argv, flags, feat))
    }
}
