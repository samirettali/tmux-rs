fn main() {
    println!("cargo::rerun-if-changed=src/grammar.lalrpop");
    lalrpop::process_root().unwrap();

    // Platform-specific linking
    #[cfg(target_os = "macos")]
    {
        println!("cargo::rustc-link-lib=ncurses");
        println!("cargo::rustc-link-lib=event");
    }

    #[cfg(target_os = "linux")]
    {
        println!("cargo::rustc-link-lib=tinfo");
        println!("cargo::rustc-link-lib=event_core");
    }
}
