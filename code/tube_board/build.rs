use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
extern crate cc;
fn main() {
    // cc::Build::new()
    //     .include("src/lib/c/include")
    //     .include("src/lib/c/cmsis_include")
    //     .file("src/lib/c/pwm.c")
    //     .compile("libpwm");
    if env::var_os("CARGO_FEATURE_RT").is_some() {
        let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
        File::create(out.join("memory.x"))
            .unwrap()
            .write_all(include_bytes!("memory.x"))
            .unwrap();
        println!("cargo:rustc-link-search={}", out.display());
        println!("cargo:rerun-if-changed=memory.x");
    }
    println!("cargo:rerun-if-changed=build.rs");
    // println!("cargo:rerun-if-changed=src/lib/c/pwm.c");
}
