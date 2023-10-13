use std::fs;
use std::io::Error;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    // set by cargo, build scripts should use this directory for output files
    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    // set by cargo's artifact dependency feature, see
    // https://doc.rust-lang.org/nightly/cargo/reference/unstable.html#artifact-dependencies
    let kernel = PathBuf::from(std::env::var_os("CARGO_BIN_FILE_KERNEL_kernel").unwrap());
    println!("cargo:rustc-env=KERNEL_BINARY={}", kernel.display());

    // create an UEFI disk image
    let uefi_path = out_dir.join("uefi.img");
    bootloader::UefiBoot::new(&kernel)
        .create_disk_image(&uefi_path)
        .unwrap();

    // create a BIOS disk image
    let bios_path = out_dir.join("bios.img");
    bootloader::BiosBoot::new(&kernel)
        .create_disk_image(&bios_path)
        .unwrap();

    // pass the disk image paths as env variables to the `main.rs`
    println!("cargo:rustc-env=UEFI_PATH={}", uefi_path.display());
    println!("cargo:rustc-env=BIOS_PATH={}", bios_path.display());

    let os_disk_dir = collect_os_disk_artifacts(&out_dir);
    let os_disk_image = create_disk_image(&out_dir, &os_disk_dir);
    println!("cargo:rustc-env=OS_DISK={}", os_disk_image.display());
}

fn create_disk_image(out_dir: &Path, os_disk_dir: &Path) -> PathBuf {
    let image_file = out_dir.join("os_disk.img").to_path_buf();
    let _ = fs::remove_file(&image_file); // if this fails, doesn't matter

    // works on my machine. TODO: use the mkfs-ext2 crate once it's ready
    let mut cmd = Command::new("mke2fs");
    cmd.arg("-d").arg(os_disk_dir.to_str().unwrap());
    cmd.arg("-m").arg("5");
    cmd.arg("-t").arg("ext2");
    cmd.arg(image_file.to_str().unwrap());
    cmd.arg("1M");

    let rc = cmd.status().unwrap();
    assert_eq!(0, rc.code().unwrap());
    image_file
}

fn collect_os_disk_artifacts(out_dir: &Path) -> PathBuf {
    let os_disk_dir = out_dir.join("os_disk");
    if os_disk_dir.exists() {
        fs::remove_dir_all(&os_disk_dir).unwrap();
    }
    fs::create_dir(&os_disk_dir).unwrap();

    // set up rootdir structure
    fs::create_dir(os_disk_dir.join("dev")).unwrap();
    fs::create_dir(os_disk_dir.join("mnt")).unwrap();

    for name in [
        // the names of the binaries are defined as artifact dependencies in the `Cargo.toml`
        "hello_world",
    ] {
        let env_name = format!("CARGO_BIN_FILE_{}_{}", name.to_uppercase(), name);
        let path = PathBuf::from(std::env::var_os(env_name).unwrap());
        copy_artifact_into_dir(&os_disk_dir, &path).unwrap();
    }

    os_disk_dir
}

fn copy_artifact_into_dir<P>(destination: P, artifact_file: P) -> Result<(), Error>
where
    P: AsRef<Path>,
{
    let dir = destination.as_ref();
    assert!(dir.exists());
    assert!(dir.is_dir());

    let file = artifact_file.as_ref();
    assert!(file.exists());
    assert!(file.is_file());

    // split off the hash of the artifact to go from 'artifact-13a6c2bf2' to 'artifact'
    let destination_file_name = file
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .and_then(|str_file_name| str_file_name.rsplit_once('-'))
        .map(|(prefix, _)| prefix)
        .unwrap();
    let destination_path = dir.join(destination_file_name);

    fs::copy(file, destination_path)?;
    Ok(())
}
