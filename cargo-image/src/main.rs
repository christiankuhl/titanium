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
    if config.build {
        Command::new("cargo").arg("kbuild").status().unwrap();
        Command::new("mkdir").arg("iso").status().unwrap();
        Command::new("mkdir").arg("iso/boot").status().unwrap();
        Command::new("mkdir").arg("iso/boot/grub").status().unwrap();
        Command::new("cp").arg("target/x86_64-titanium/release/titanium").arg("iso/boot/titanium.bin").status().unwrap();
        let mut cfg = File::create("iso/boot/grub/grub.cfg").expect("Failed to create grub.cfg");
        cfg.write_all(b"set timeout=0\nset default=0\n\nmenuentry \"Titanium OS\" {\nmultiboot2 /boot/titanium.bin\nboot\n}").unwrap();
        Command::new("grub-mkrescue").arg("--output=mykernel.iso").arg("iso").status().unwrap();
        Command::new("rm").arg("-rf").arg("iso").status().unwrap();
    }
    if config.run {
        Command::new("qemu-system-x86_64").arg("--cdrom").arg("mykernel.iso").status().unwrap();
    }
}

struct Config {
    run: bool,
    build: bool,
}

impl Config {
    fn new(args: &[String]) -> Result<Self, String> {
        if args.len() < 3 {
            return Err("Not enough arguments!".to_string())
        }
        let mut config = Config { run: false, build: false };
        match args[2].to_lowercase().as_str() {
            "build" => {
                config.build = true;
                return Ok(config)
            }
            "run" => {
                config.build = true;
                config.run = true;
                return Ok(config)
            }
            arg => return Err(format!("Unknown argument: {}", arg.clone()))
        }
    }
}