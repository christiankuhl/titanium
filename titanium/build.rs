fn main() {
    cc::Build::new().file("src/loader.s").debug(true).compile("loader");
    cc::Build::new().file("src/long_mode_init.s").debug(true).compile("long_mode");
    println!("cargo:rustc-link-arg=--nmagic");
    println!("cargo:rustc-link-arg=-Tlinker.ld");
}