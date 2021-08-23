use std::process::Command;
use std::env::args;

fn main() {
    Command::new("cargo").arg("build").status().expect("Build failed!");
    let kernel_binary = "../target/x86_64-titanium/debug/titanium";
    toolchain::build_bootimage(kernel_binary, false);
    let mut add_args = args();
    add_args.next();
    add_args.next();
    toolchain::start_qemu(kernel_binary, false, true, add_args)
}