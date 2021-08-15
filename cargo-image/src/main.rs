use std::process::Command;
use std::fs::File;
use std::io::Write;

fn main() {
    Command::new("mkdir").arg("iso").status().unwrap();
    Command::new("mkdir").arg("iso/boot").status().unwrap();
    Command::new("mkdir").arg("iso/boot/grub").status().unwrap();
    Command::new("cp").arg("target/x86_64-titanium/debug/titanium").arg("iso/boot/titanium.bin").status().unwrap();
    let mut cfg = File::create("iso/boot/grub/grub.cfg").expect("Failed to create grub.cfg");
    cfg.write_all(b"set timeout=0\nset default=0\n\nmenuentry \"Titanium OS\" {\nmultiboot /boot/titanium.bin\nboot\n}").unwrap();
    Command::new("grub-mkrescue").arg("--output=mykernel.iso").arg("iso").status().unwrap();
	Command::new("rm").arg("-rf").arg("iso").status().unwrap();
}