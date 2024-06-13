extern crate bootloader;
extern crate fs_extra;

use std::fs;
use std::io::Error;
use std::path::{Path, PathBuf};
use std::process::Command;

use bootloader::BootConfig;

fn main() {
    // set by cargo, build scripts should use this directory for output files
    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    // set by cargo's artifact dependency feature, see
    // https://doc.rust-lang.org/nightly/cargo/reference/unstable.html#artifact-dependencies
    let kernel = PathBuf::from(std::env::var_os("CARGO_BIN_FILE_KERNEL_kernel").unwrap());
    println!("cargo:rustc-env=KERNEL_BINARY={}", kernel.display());

    let mut boot_config = BootConfig::default();
    boot_config.frame_buffer_logging = false;
    boot_config.serial_logging = false;

    // create an UEFI disk image
    let uefi_path = out_dir.join("uefi.img");
    bootloader::UefiBoot::new(&kernel)
        .set_boot_config(&boot_config)
        .create_disk_image(&uefi_path)
        .unwrap();

    // pass the disk image paths as env variables to the `main.rs`
    println!("cargo:rustc-env=UEFI_PATH={}", uefi_path.display());

    // create a disk image for each test kernel
    for test_kernel in fs::read_dir("tests")
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|e| {
            e.is_dir()
                && e.file_name()
                .is_some_and(|f| f.to_str().unwrap().starts_with("test_kernel_"))
        })
        .map(|e| {
            e.file_name()
                .map(|f| f.to_str().unwrap().to_string())
                .unwrap()
        })
    {
        let test_kernel_binary_path = PathBuf::from(
            std::env::var_os(format!(
                "CARGO_BIN_FILE_{}_{}",
                test_kernel.to_uppercase(),
                test_kernel
            ))
                .unwrap(),
        );
        let test_kernel_path = out_dir.join(format!("{test_kernel}.img"));
        bootloader::UefiBoot::new(&test_kernel_binary_path)
            .set_boot_config(&boot_config)
            .create_disk_image(&test_kernel_path)
            .unwrap();
        println!(
            "cargo:rustc-env={}_PATH={}",
            test_kernel.to_uppercase(),
            test_kernel_path.display()
        );
    }

    let os_disk_dir = build_os_disk(&out_dir);
    let os_disk_image = create_ext2_image(&out_dir, &os_disk_dir);
    println!("cargo:rustc-env=OS_DISK={}", os_disk_image.display());
}

fn create_ext2_image(out_dir: &Path, os_disk_dir: &Path) -> PathBuf {
    let image_file = out_dir.join("os_disk.img").to_path_buf();
    let _ = fs::remove_file(&image_file); // if this fails, doesn't matter

    // works on my machine. TODO: use the mkfs-ext2 crate once it's ready
    let mut cmd = Command::new("mke2fs");
    cmd.arg("-d").arg(os_disk_dir.to_str().unwrap());
    cmd.arg("-m").arg("5");
    cmd.arg("-t").arg("ext2");
    cmd.arg(image_file.to_str().unwrap());
    cmd.arg("5M");

    let rc = cmd.status().unwrap();
    assert_eq!(0, rc.code().unwrap());
    image_file
}

fn build_os_disk(out_dir: &Path) -> PathBuf {
    let os_disk_dir = out_dir.join("os_disk");
    if os_disk_dir.exists() {
        fs::remove_dir_all(&os_disk_dir).unwrap();
    }

    // copy the os_disk dir
    let os_disk_src_dir =
        PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("os_disk");
    fs_extra::dir::copy(os_disk_src_dir, out_dir, &Default::default()).unwrap();

    // remove all .gitkeep files anywhere in `os_disk_dir`
    fs::read_dir(&os_disk_dir)
        .unwrap()
        .map(Result::unwrap)
        .filter_map(|e| {
            let path = e.path();
            if path.is_file() && path.file_name().unwrap() == ".gitkeep" {
                return Some(path);
            }
            None
        })
        .for_each(|e| {
            fs::remove_file(e).unwrap();
        });

    let copy_bindep = |bindep_name: &str, dest: &str| {
        let env_name = format!(
            "CARGO_BIN_FILE_{}_{}",
            bindep_name.to_uppercase(),
            bindep_name
        );
        let path = PathBuf::from(std::env::var_os(&env_name).unwrap_or_else(|| {
            panic!(
                "env var {} is not set, did you add '{}' as a bindep?",
                env_name, bindep_name
            )
        }));
        let dest = &dest[1..]; // remove the leading '/'
        if dest.is_empty() {
            copy_artifact_into_dir(&os_disk_dir, &path).unwrap();
        } else {
            copy_artifact_into_dir(os_disk_dir.join(dest), path).unwrap();
        }
    };

    copy_bindep("hello_world", "/bin");
    copy_bindep("window_server", "/bin");

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
