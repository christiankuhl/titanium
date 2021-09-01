use serde_derive::Deserialize;
use std::fs::{read_to_string, File};
use std::io::{Result, Write};
use std::process::{exit, Command, ExitStatus, Stdio};
use toml;

pub fn build_bootimage(kernel_binary: &str, test: bool) {
    if !test {
        println!("\nProducing bootable image for {}", kernel_binary);
    }
    Command::new("mkdir").arg("iso").status().unwrap();
    Command::new("mkdir").arg("iso/boot").status().unwrap();
    Command::new("mkdir").arg("iso/boot/grub").status().unwrap();
    Command::new("cp").arg(kernel_binary).arg("iso/boot/titanium.bin").status().unwrap();
    let mut cfg = File::create("iso/boot/grub/grub.cfg").expect("Failed to create grub.cfg");
    cfg.write_all(b"set timeout=0\nset default=0\n\nmenuentry \"Titanium OS\" {\nmultiboot2 /boot/titanium.bin\nboot\n}")
        .unwrap();
    Command::new("grub-mkrescue")
        .stderr(if !test { Stdio::inherit() } else { Stdio::null() })
        .arg("--output=titanium.iso")
        .arg("iso")
        .status()
        .unwrap();
    Command::new("rm").arg("-rf").arg("iso").status().unwrap();
}

pub fn start_qemu(kernel_binary: &str, test: bool, debug: bool, add_args: Vec<&str>) {
    let mut args = vec![
        "--cdrom",
        "titanium.iso",
        "-m",
        "1G",
        "-drive",
        "id=disk,file=myimage.img,if=none,format=raw",
        "-device",
        "ahci,id=ahci",
        "-device",
        "ide-hd,drive=disk,bus=ahci.0",
    ];
    if debug {
        args.extend(vec!["-s", "-S"])
    } else {
        args.push("-enable-kvm")
    };
    if test {
        args.extend(vec!["-device", "isa-debug-exit,iobase=0xf4,iosize=0x04", "-serial", "stdio", "-display", "none"])
    }
    args.extend(add_args);
    let mut child = Command::new("qemu-system-x86_64").args(args).spawn().expect("Failed to run QEMU");
    if debug {
        Command::new("rust-gdb").arg(&kernel_binary).arg("-ex").arg("source debug.gdb").status().unwrap();
    }
    let exit_code = child.wait().expect("Something went horribly wrong!").code().expect("Process killed by signal!");
    exit(exit_code - 33)
}

pub fn is_test_run(kernel_binary: &str) -> bool {
    kernel_binary.contains("/debug/deps/")
}

#[derive(Debug, Deserialize)]
pub struct IntegrationTests {
    test: Vec<TestConfig>,
}

impl Iterator for IntegrationTests {
    type Item = TestConfig;

    fn next(&mut self) -> Option<Self::Item> {
        self.test.pop()
    }
}

#[derive(Debug, Deserialize)]
pub struct TestConfig {
    name: String,
    features: Option<Vec<String>>,
}

pub fn integration_tests() -> IntegrationTests {
    let file = read_to_string("tests/tests.toml").expect("Could not find tests/test.toml");
    let result = toml::from_str(&file).expect("Could not parse tests/result.toml");
    result
}

fn test(config: TestConfig, add_args: &Vec<&str>) {
    let features = match config.features {
        Some(list) => list.join(","),
        None => "".to_owned(),
    };
    let mut args = vec!["test", "--test"];
    args.push(&config.name);
    if features.len() > 0 {
        args.extend(vec!["--features", &features]);
    }
    args.extend(add_args);
    let status = Command::new("cargo").stderr(Stdio::null()).args(args).status();
    handle_cargo_status(status);
}

fn handle_cargo_status(status: Result<ExitStatus>) {
    let code = status.unwrap().code().unwrap();
    if code > 0 {
        println!("Compilation failed!");
        exit(code);
    }
}

pub fn execute_test_suite(add_args: Vec<&str>) {
    let mut status = Command::new("cargo")
        .stderr(Stdio::null())
        .arg("test")
        .arg("--lib")
        .arg("--features")
        .arg("qemu_test")
        .args(&add_args)
        .status();
    handle_cargo_status(status);
    status = Command::new("cargo")
        .arg("test")
        .stderr(Stdio::null())
        .arg("--bin")
        .arg("titanium")
        .arg("--features")
        .arg("qemu_test")
        .status();
    handle_cargo_status(status);
    banner("Kernel integration tests...");
    println!();
    for int_test in integration_tests() {
        test(int_test, &add_args);
    }
    println!()
}

pub fn banner(title: &str) {
    use AnsiColors::*;
    println!("{}", Yellow.text("\n╔══════════════════════════════════════════════════════════════════════════════╗"));
    println!("{}", Yellow.text(&format!("║ {:<76} ║", title)));
    println!("{}", Yellow.text("╚══════════════════════════════════════════════════════════════════════════════╝"));
}

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum AnsiColors {
    Reset = 0,
    Black = 30,
    Red = 31,
    Green = 32,
    Yellow = 33,
    Blue = 34,
    Magenta = 35,
    Cyan = 36,
    White = 37,
}

impl AnsiColors {
    pub fn text(&self, text: &str) -> String {
        format!("\u{001b}[{}m{}\u{001b}[{}m", *self as u8, text, Self::Reset as u8)
    }
}
