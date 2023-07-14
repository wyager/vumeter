use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
extern crate cc;

static CFLAGS: &[&str] = &[
    "-c", 
    "-Wall", 
    "-MMD", 
    "-g", 
    "-O2", 
    "-ffunction-sections", 
    "-fdata-sections", 
    "-mcpu=cortex-m7", 
    "-mthumb", 
    "-mfloat-abi=hard", 
    "-mfpu=fpv5-d16", 
    "-std=gnu11",
];
/// Preprocessor flags
static CPPFLAGS: &[&str] = &[
    "-D__IMXRT1062__",
    "-DARDUINO_TEENSY40",
    // TODO figure out how to handle / alias these
    "-DFLASHMEM=__attribute__((section(\".flashmem\")))",
    "-DPROGMEM=__attribute__((section(\".progmem\")))",
    "-DDMAMEM=__attribute__ ((section(\".dmabuffers\"), used))",
];
/// The C compiler
static CC: &str = "arm-none-eabi-gcc";
/// The archiver
static AR: &str = "arm-none-eabi-gcc-ar";

fn main() {
    // Path to your custom linker script
    let linker_script_path = Path::new("linker_t4.x");
    // Path to the output file
    let out_dir = env::var("OUT_DIR").unwrap();
    let output_path = Path::new(&out_dir).join("link.x");
    // Copy the linker script to the output directory
    std::fs::copy(linker_script_path, &output_path).unwrap();

    let mut builder = cc::Build::new();
    builder
       .include("src/lib/c/cores/teensy4")
       .include("src/lib/c/audio")
       .file("src/lib/c/cores/teensy4/digital.c")
       .file("src/lib/c/spdif.c")
       .file("src/lib/c/timer.c")
       .file("src/lib/c/switch.c")
       .file("src/lib/c/led.c")
       .file("src/lib/c/uart.c");
    builder.compiler(CC);
    builder.archiver(AR);
    builder.no_default_flags(true);
    for &flag in CFLAGS.iter().chain(CPPFLAGS.iter()) {
        builder.flag(flag);
    }
    builder.compile("libteensy");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/lib/c/spdif.c");
    println!("cargo:rerun-if-changed=src/lib/c/led.c");
    println!("cargo:rerun-if-changed=src/lib/c/uart.c");
    println!("cargo:rerun-if-changed=src/lib/c/timer.c");
    println!("cargo:rerun-if-changed=src/lib/c/switch.c");
    println!("cargo:rerun-if-changed=src/lib/c/cores/teensy4/digital.c");

    // Set the environment variable for the linker script
    println!("cargo:rustc-link-search={}", output_path.parent().unwrap().display());
    println!("cargo:rustc-linker=lld");
    println!("cargo:rerun-if-changed={}", linker_script_path.display());

}
