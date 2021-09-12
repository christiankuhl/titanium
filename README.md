# Titanium

## What is it?

This is a toy operating system I am writing solely for educational and recreational purposes.

## What can it do?

It's state is currently very rudimentary: We can 
* boot into long mode 
* print keyboard and mouse input to the VGA text buffer
* allocate memory on the heap and have paging working
* enumerate PCI devices
* do elementary task switching
* identify SATA drives.

## Where are you going with this?

Probably nowhere ;o)

The idea for Titanium is to combine things I like (Rust for performance sensitive programming, Python for everything else)
with a thing I miss: The immediacy of interacting programmatically with the machine that I got from the computers I grew
up with.

Hence, the long-term goal for this project is to have a rudimentary working OS that boots directly into a Python REPL.

## Why the silly name?

No reason at all, in fact, the name comes from a website generating random meaningless project names.

## Credit where credit is due...

This would not go anywhere if it weren't for the following resources:
* The [OSDev.org wiki](https://wiki.osdev.org/Main_Page)
* Philipp Oppermann's excellent [blog](https://os.phil-opp.com/)
* This amazing [youtube playlist](https://www.youtube.com/watch?v=1rnA6wpF0o4&list=PLHh55M_Kq4OApWScZyPl5HhgsTJS9MZ6M)

## How to install

... oh, you're still here...

Now, if you absolutely must:

0. Install prerequisites
* [QEMU](https://www.qemu.org/) as an emulator
* [GRUB](https://www.gnu.org/software/grub/) to create a bootable image
* [gdb](https://www.gnu.org/software/gdb/) for debugging purposes.

1. Clone the repository
```
git clone https://github.com/christiankuhl/titanium
```
2. Build the toolchain (adds new `cargo` subcommands `image` (to create a bootable image) and `debug` (to debug the kernel with `gdb`))
```
cd titanium
cargo toolchain
```
3. Build/run the kernel itself
```
cd kernel
cargo run
```
