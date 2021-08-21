use std::process::{Command, Stdio, exit};
use std::fs::File;
use std::io::Write;

pub fn build_bootimage(kernel_binary: &str, test: bool) {
    if !test {
        println!("Producing bootable image for {}", kernel_binary);
    }
    Command::new("mkdir").arg("iso").status().unwrap();
    Command::new("mkdir").arg("iso/boot").status().unwrap();
    Command::new("mkdir").arg("iso/boot/grub").status().unwrap();
    Command::new("cp").arg(kernel_binary).arg("iso/boot/titanium.bin").status().unwrap();
    let mut cfg = File::create("iso/boot/grub/grub.cfg").expect("Failed to create grub.cfg");
    cfg.write_all(b"set timeout=0\nset default=0\n\nmenuentry \"Titanium OS\" {\nmultiboot2 /boot/titanium.bin\nboot\n}")
        .unwrap();
    Command::new("grub-mkrescue")
        .stderr(
            if !test {
                Stdio::inherit()
            } else {
                Stdio::null()
            })
        .arg("--output=titanium.iso")
        .arg("iso").status().unwrap();
    Command::new("rm").arg("-rf").arg("iso").status().unwrap();
}

pub fn start_qemu(kernel_binary: &str, test: bool, debug: bool) {
    let mut args = vec![
        "--cdrom", "titanium.iso",
        "-enable-kvm",
        "-m", "1G",
    ];
    if debug { args.extend(vec!["-s", "-S"]) };
    if test {
        args.extend(
            vec![
            "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04", 
            "-serial", "stdio",
            "-display", "none"
            ]) 
    }
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