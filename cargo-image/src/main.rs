use std::process;
use std::process::Command;
use std::fs::File;
use std::io::Write;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = Config::new(&args).unwrap_or_else(|err| {
        println!("Error: {}", err);
        process::exit(1);
    });
    let profile = if config.debug { "debug" } else { "release" };
    let kernel_binary = format!("target/x86_64-titanium/{}/titanium", profile);
    if config.build {
        let mut cargo = Command::new("cargo");
        let build_cmd = if config.debug { 
            cargo.arg("kbuild")
        } else {
            cargo.arg("kbuild").arg("--release")
        };
        build_cmd.status().expect("Build failed!");
        Command::new("mkdir").arg("iso").status().unwrap();
        Command::new("mkdir").arg("iso/boot").status().unwrap();
        Command::new("mkdir").arg("iso/boot/grub").status().unwrap();
        Command::new("cp").arg(&kernel_binary).arg("iso/boot/titanium.bin").status().unwrap();
        let mut cfg = File::create("iso/boot/grub/grub.cfg").expect("Failed to create grub.cfg");
        cfg.write_all(b"set timeout=0\nset default=0\n\nmenuentry \"Titanium OS\" {\nmultiboot2 /boot/titanium.bin\nboot\n}").unwrap();
        Command::new("grub-mkrescue").arg("--output=mykernel.iso").arg("iso").status().unwrap();
        Command::new("rm").arg("-rf").arg("iso").status().unwrap();
    }
    if config.run {
        let mut qemu = Command::new("qemu-system-x86_64");
        if !config.debug {
            qemu.arg("--cdrom").arg("mykernel.iso")
                .arg("-m").arg("1G").status().unwrap();
        } else {
            qemu.arg("--cdrom").arg("mykernel.iso")
                .arg("-m").arg("1G").arg("-s").arg("-S")
                .spawn().expect("Failed to run QEMU");
            Command::new("rust-gdb").arg(&kernel_binary).arg("-ex").arg("source debug.gdb")
                .status().unwrap();
        }
    }
}

struct Config {
    run: bool,
    build: bool,
    debug: bool,
}

impl Config {
    fn new(args: &[String]) -> Result<Self, String> {
        if args.len() < 3 {
            return Err("Not enough arguments!".to_string())
        }
        let mut config = Config { run: false, build: false, debug: false };
        match args[2].to_lowercase().as_str() {
            "build" => {
                config.build = true;
            }
            "run" => {
                config.build = true;
                config.run = true;
            }
            arg => return Err(format!("Unknown argument: {}", arg.clone()))
        }
        config.debug = args.iter().any(|arg| arg.to_lowercase() == "--debug");
        Ok(config)
    }
}