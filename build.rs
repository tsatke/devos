use std::path::PathBuf;

fn main() {
    // set by cargo, build scripts should use this directory for output files
    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    // set by cargo's artifact dependency feature, see
    // https://doc.rust-lang.org/nightly/cargo/reference/unstable.html#artifact-dependencies
    let kernel = PathBuf::from(std::env::var_os("CARGO_BIN_FILE_KERNEL_kernel").unwrap());
    let test_kernel =
        PathBuf::from(std::env::var_os("CARGO_BIN_FILE_KERNEL_TEST_kernel_test").unwrap());

    // create an UEFI disk image
    let uefi_path = out_dir.join("uefi.img");
    bootloader::UefiBoot::new(&kernel)
        .create_disk_image(&uefi_path)
        .unwrap();

    // create an UEFI disk image for the test kernel
    let uefi_test_path = out_dir.join("uefi_test.img");
    bootloader::UefiBoot::new(&test_kernel)
        .create_disk_image(&uefi_test_path)
        .unwrap();

    // create a BIOS disk image
    let bios_path = out_dir.join("bios.img");
    bootloader::BiosBoot::new(&kernel)
        .create_disk_image(&bios_path)
        .unwrap();

    // pass the disk image paths as env variables to the `main.rs`
    println!("cargo:rustc-env=UEFI_PATH={}", uefi_path.display());
    println!(
        "cargo:rustc-env=UEFI_TEST_PATH={}",
        uefi_test_path.display()
    );
    println!("cargo:rustc-env=BIOS_PATH={}", bios_path.display());
}
