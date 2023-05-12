<div align="center">

# DevOS

[![build](https://github.com/tsatke/devos/actions/workflows/build.yml/badge.svg)](https://github.com/tsatke/devos/actions/workflows/build.yml)
[![lint](https://github.com/tsatke/devos/actions/workflows/lint.yml/badge.svg)](https://github.com/tsatke/devos/actions/workflows/lint.yml)

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
  - [ ] VGA
  - [ ] NVMe
  - [ ] Networking
- [ ] Virtual File System with EXT2 implementation
- [ ] Preemptive multitasking
- [ ] Interactive shell
- [ ] Test framework

### Build and Run

To run the kernel in QEMU

```plain
cargo run
```
