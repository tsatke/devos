<div align="center">

# DevOS

[![build](https://github.com/tsatke/devos/actions/workflows/build.yml/badge.svg)](https://github.com/tsatke/devos/actions/workflows/build.yml)
[![lint](https://github.com/tsatke/devos/actions/workflows/lint.yml/badge.svg)](https://github.com/tsatke/devos/actions/workflows/lint.yml)

An operating system and kernel developed with the Rust programming language, with an emphasis on usability for developers who use it.

[Requirements](#requirements) •
[Build and Run](#build-and-run)

</div>

### Requirements

* QEMU
* [`rustup`](https://rustup.rs)

### Build and Run

To run the kernel in QEMU

```plain
cargo run
```

To run the tests

```plain
cargo test
```

#### What else can I do?

```plain
cargo run -- --help
```
