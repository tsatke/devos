<div align="center">

# DevOS

[![build](https://github.com/tsatke/devos/actions/workflows/build.yml/badge.svg)](https://github.com/tsatke/devos/actions/workflows/build.yml)

An operating system and kernel developed with the Rust programming language, with an emphasis on usability for developers who use it.

[Requirements](#requirements) •
[Features](#features) •
[Build and Run](#build-and-run)

</div>

### Requirements

* QEMU
* [`rustup`](https://rustup.rs)

### Features
Check out the next [milestone](https://github.com/tsatke/devos/milestone/1) for progress.
- [x] Heap
- [x] Syscalls
- [ ] Drivers
  - [x] PCI driver
  - [x] IDE driver (no ATA)
  - [x] VGA
  - [ ] NVMe
  - [ ] Networking
- [x] Virtual File System with EXT2 implementation (read-only for now)
- [x] Preemptive multitasking (processes & threads)
- [ ] Interactive shell
- [x] Test framework

### Build and Run

To run the kernel in QEMU

```plain
cargo run
```

### Running tests

To run tests with QEMU, run

```plain
cargo test
```

This will run kernel unit tests, as well as all `test_kernels`.

### Debugging

To debug the kernel in QEMU, run

```plain
cargo run -- --debug
```

and then connect to the QEMU instance with `lldb`:

```plain
lldb -s debug.lldb
```
