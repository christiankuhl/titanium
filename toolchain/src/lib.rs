use serde_derive::Deserialize;
use std::fs::read_to_string;
use std::io::Result;
use std::process::{exit, Command, ExitStatus, Stdio};
use toml;

pub const IMAGE_NAME: &str = "titanium.img";
const MOUNT_DIR: &str = "img";

fn cleanup(loopback_device: &str) {
    Command::new("sudo")
        .arg("umount")
        .arg(MOUNT_DIR)
        .status()
        .or_else(|x| {
            Command::new("sleep").arg("1").status().unwrap();
            Command::new("sudo").arg("umount").arg(MOUNT_DIR).status().unwrap();
            Err(x)
        })
        .unwrap();
    Command::new("sudo").arg("rmdir").arg(MOUNT_DIR).status().expect("Failed to delete mount directory!");
    Command::new("sudo").arg("losetup").arg("-d").arg(loopback_device).output().expect("Failed to clean up loopback device!");
}

fn mount_disk_image(loopback_device: &str) {
    Command::new("mkdir").arg("-p").arg(MOUNT_DIR).status().unwrap();
    Command::new("sudo")
        .arg("mount")
        .arg(format!("{}p1", loopback_device))
        .arg(format!("{}/", MOUNT_DIR))
        .status()
        .expect("Unable to mount image!");
}

fn install_grub(loopback_device: &str) {
    Command::new("sudo")
        .arg("grub-install")
        .arg(format!("--boot-directory={}/boot", MOUNT_DIR))
        .arg("--target=i386-pc")
        .arg("--modules=ext2 part_msdos")
        .arg(loopback_device)
        .status()
        .expect("Unable to install GRUB!");
}

fn create_new_disk_image(image_name: &str, size: usize) -> String {
    // Create the raw image
    Command::new("qemu-img")
        .arg("create")
        .arg("-q")
        .arg("-f")
        .arg("raw")
        .arg(image_name)
        .arg(format!("{}M", size / (1024 * 1024)))
        .status()
        .unwrap();
    // Set up loopback device
    let loopback_device = set_up_loopback_device(image_name);
    println!("Set up loopback device: {}", &loopback_device);
    // Create partition table
    let parted_args =
        vec!["mklabel", "msdos", "mkpart", "primary", "ext2", "1MiB", "100%", "-a", "minimal", "set", "1", "boot", "on"];
    println!("Creating partition table");
    Command::new("sudo")
        .arg("parted")
        .arg("-s")
        .arg(&loopback_device)
        .args(parted_args)
        .status()
        .expect("Failed to set up partition table!");
    // Destroy old filesystem (if any)
    Command::new("sudo")
        .arg("dd")
        .arg("if=/dev/zero")
        .arg(format!("of={}p1", &loopback_device))
        .arg("bs=1M")
        .arg("count=1")
        .arg("status=none")
        .status()
        .expect(&format!("Could not destroy old filesystem on {}", &loopback_device));
    // Build new filesystem
    Command::new("sudo")
        .arg("mke2fs")
        .arg("-q")
        .arg("-I")
        .arg("128")
        .arg(format!("{}p1", &loopback_device))
        .status()
        .expect(&format!("Could not build new ext2 filesystem on {}", &loopback_device));
    loopback_device
}

fn set_up_loopback_device(image_name: &str) -> String {
    let losetup =
        Command::new("sudo").arg("losetup").arg("--find").arg("--partscan").arg("--show").arg(image_name).output().unwrap();
    let loopback_device = String::from_utf8(losetup.stdout).expect("Failed to set up loopback device!").replace("\n", "");
    loopback_device
}

fn create_root_filesystem() {
    Command::new("sudo").arg("mkdir").arg("-p").arg(format!("{}/boot/grub", MOUNT_DIR)).status().unwrap();
    Command::new("sudo").arg("chmod").arg("700").arg(format!("{}/boot", MOUNT_DIR)).status().unwrap();
    for dir in vec!["bin", "etc", "proc", "mnt", "tmp", "boot", "mod", "var/run"] {
        Command::new("sudo").arg("mkdir").arg("-p").arg(format!("{}/{}", MOUNT_DIR, dir)).status().unwrap();
    }
    Command::new("sudo")
        .arg("cp")
        .arg("../toolchain/grub.cfg")
        .arg(format!("{}/boot/grub/grub.cfg", MOUNT_DIR))
        .status()
        .unwrap();
}

fn disk_usage() -> usize {
    let du_output = Command::new("du").arg("-s").arg("--apparent-size").arg("../target/").output().unwrap();
    let size = usize::from_str_radix(
        String::from_utf8(du_output.stdout)
            .expect("Unable get output from 'du'")
            .split("\t")
            .next()
            .expect("Malformed 'du' output"),
        10,
    )
    .expect("Unable to parse 'du' output");
    println!("Size needed: {} MB", size / (1024 * 1024) + 500);
    size + 500 * 1024 * 1024
}

pub struct DiskImage {
    loopback_device: String,
    mounted: bool,
    debug: bool,
}

impl DiskImage {
    pub fn create(debug: bool) -> Self {
        println!("Creating new image file {}...", IMAGE_NAME);
        let size = disk_usage();
        let loopback_device = create_new_disk_image(IMAGE_NAME, size);
        mount_disk_image(&loopback_device);
        create_root_filesystem();
        install_grub(&loopback_device);
        Self { loopback_device, mounted: true, debug }
    }
    pub fn from_existing(image_file: &str, debug: bool) -> Self {
        if debug {
            println!("Using existing image file {}...", image_file);
        }
        let loopback_device = set_up_loopback_device(image_file);
        mount_disk_image(&loopback_device);
        Self { loopback_device, mounted: true, debug }
    }
    pub fn update(&self, file: &str, with_file: &str, owner: Option<&str>, permissions: Option<&str>, debug: bool) {
        if debug {
            println!("Updating {} with {}...", file, with_file);
        }
        let tgt = format!("{}/{}", MOUNT_DIR, file);
        Command::new("sudo").arg("cp").arg(with_file).arg(&tgt).status().unwrap();
        if let Some(owner) = owner {
            Command::new("sudo").arg("chown").arg(owner).arg(&tgt).status().unwrap();
        }
        if let Some(permissions) = permissions {
            Command::new("sudo").arg("chmod").arg(permissions).arg(&tgt).status().unwrap();
        }
    }
}

impl Drop for DiskImage {
    fn drop(&mut self) {
        if self.mounted {
            if self.debug {
                println!("Unmounting {}...", self.loopback_device);
            }
            Command::new("sleep").arg("1").status().unwrap();
            cleanup(&self.loopback_device);
        }
    }
}

pub fn start_qemu(kernel_binary: &str, test: bool, debug: bool, add_args: Vec<&str>) {
    let mut args = vec![
        "-m",
        "1G",
        "-cpu",
        "host",
        "-drive",
        "id=disk,file=titanium.img,if=none,index=0,format=raw",
        "-device",
        "ahci,id=ahci",
        "-device",
        "ide-hd,drive=disk,bus=ahci.0",
        "-enable-kvm",
    ];
    if debug {
        args.extend(vec!["-s", "-S"]);
    }
    if test {
        args.extend(vec!["-device", "isa-debug-exit,iobase=0xf4,iosize=0x04", "-serial", "stdio", "-display", "none"])
    }
    args.extend(add_args);
    println!("{:?}", &args);
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
