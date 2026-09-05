use std::{env, fs, path::PathBuf};

fn main() {
    let out = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR is set by Cargo"));
    let memory = if env::var_os("CARGO_FEATURE_LPC1115").is_some() {
        "memory-lpc1115.x"
    } else {
        "memory.x"
    };
    fs::copy(memory, out.join("memory.x")).expect("copy memory.x");
    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rerun-if-changed={memory}");
}
