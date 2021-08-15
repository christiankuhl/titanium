use std::process::Command;

fn main() {
    Command::new("alias").arg("cargo-image='target/release/cargo-image'").status().unwrap();
}