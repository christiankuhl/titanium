use std::env;
use std::env::Args;

fn main() {
    let mut args = env::args();
    args.next();
    match args.next().as_deref() {
        Some("image") => {
            // Direct call of cargo image
            match args.next().as_deref() {
                Some("test") => {
                    let add_args = args.collect::<Vec<String>>();
                    let add_args: Vec<&str> = add_args.iter().map(|s| &**s).collect();
                    toolchain::execute_test_suite(add_args)
                }
                Some(kernel_binary) => build_and_run(kernel_binary, args),
                None => {
                    panic!("Not enough arguments provided!")
                }
            }
        }
        Some(kernel_binary) => build_and_run(kernel_binary, args),
        None => {
            panic!("Not enough arguments provided!")
        }
    }
}

fn build_and_run(kernel_binary: &str, args: Args) {
    let test = toolchain::is_test_run(kernel_binary);
    toolchain::build_bootimage(kernel_binary, test);
    let add_args = args.collect::<Vec<String>>();
    let add_args: Vec<&str> = add_args.iter().map(|s| &**s).collect();
    toolchain::start_qemu(kernel_binary, test, false, add_args)
}
