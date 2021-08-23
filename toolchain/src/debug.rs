use std::process::Command;
use std::env::args;

fn main() {
    Command::new("cargo").arg("build").status().expect("Build failed!");
    let kernel_binary = "../target/x86_64-titanium/debug/titanium";
    toolchain::build_bootimage(kernel_binary, false);
    toolchain::start_qemu(kernel_binary, false, true, args())
}