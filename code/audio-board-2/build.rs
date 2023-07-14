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
        let mut builder = cc::Build::new();
        builder
       .include("src/bin/c/cores/teensy4")
       .include("src/bin/c/audio")
       .file("src/bin/c/cores/teensy4/digital.c")
       .file("src/bin/c/spdif.c")
       .file("src/bin/c/timer.c")
       .file("src/bin/c/switch.c")
       .file("src/bin/c/led.c")
       .file("src/bin/c/uart.c");
    builder.compiler(CC);
    builder.archiver(AR);
    builder.no_default_flags(true);
    for &flag in CFLAGS.iter().chain(CPPFLAGS.iter()) {
        builder.flag(flag);
    }
    builder.compile("libteensy");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/bin/c/spdif.c");
    println!("cargo:rerun-if-changed=src/bin/c/led.c");
    println!("cargo:rerun-if-changed=src/bin/c/uart.c");
    println!("cargo:rerun-if-changed=src/bin/c/timer.c");
    println!("cargo:rerun-if-changed=src/bin/c/switch.c");
    println!("cargo:rerun-if-changed=src/bin/c/cores/teensy4/digital.c");
}