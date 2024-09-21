<div align="center">

# DevOS

[![project chat](https://img.shields.io/badge/zulip-join_chat-brightgreen.svg)](https://devos.zulipchat.com)
[![build](https://github.com/tsatke/devos/actions/workflows/build.yml/badge.svg)](https://github.com/tsatke/devos/actions/workflows/build.yml)

An operating system and kernel developed with the Rust programming language.

[Requirements](#requirements) •
[Features](#features) •
[Build and Run](#build-and-run)

</div>

### Requirements

* QEMU (for running and testing the kernel)
* [`rustup`](https://rustup.rs)
* `cc` (probably already available on your system)
* `mke2fs` for now

With those installed, you can simply run `cargo build` to build an image.
If you also have QEMU installed, `cargo run` and `cargo test` also will work.

### Features

Check out the next [milestone](https://github.com/tsatke/devos/milestone/1) for progress.

- [x] Heap
- [x] Syscalls
- [ ] Drivers
    - [x] PCI driver
    - [x] IDE driver (no ATA)
    - [x] VGA
    - [x] xHCI
    - [ ] USB
    - [ ] NVMe
    - [ ] Networking
- [x] Virtual File System with EXT2 implementation
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
