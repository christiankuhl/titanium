use std::env;

fn main() {
    let mut args = env::args();
    args.next();
    let kernel_binary = args.next().unwrap();
    let test = toolchain::is_test_run(kernel_binary.as_str());
    toolchain::build_bootimage(kernel_binary.as_str(), test);
    toolchain::start_qemu(kernel_binary.as_str(), test, false, args)
}