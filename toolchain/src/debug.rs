use std::env::args;
use std::process::Command;

fn main() {
    Command::new("cargo").arg("build").status().expect("Build failed!");
    let kernel_binary = "../target/x86_64-titanium/debug/titanium";
    toolchain::build_bootimage(kernel_binary, false);
    let mut add_args = args();
    add_args.next();
    add_args.next();
    let add_args = add_args.collect::<Vec<String>>();
    let add_args: Vec<&str> = add_args.iter().map(|s| &**s).collect();
    toolchain::start_qemu(kernel_binary, false, true, add_args)
}
