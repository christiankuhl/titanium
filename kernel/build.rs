fn main() {
    cc::Build::new().file("src/multiboot/asm/loader.s").flag("-fno-asynchronous-unwind-tables").compile("loader");
    cc::Build::new().file("src/multiboot/asm/long_mode_init.s").flag("-fno-asynchronous-unwind-tables").compile("long_mode");
    println!("cargo:rustc-link-arg=--nmagic");
    println!("cargo:rustc-link-arg=-Tlinker.ld");
}
