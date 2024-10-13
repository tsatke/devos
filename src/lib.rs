use rand::distributions::Alphanumeric;
use rand::Rng;

// these are set in build.rs at build time
pub const UEFI_PATH: &str = env!("UEFI_PATH");
pub const KERNEL_BINARY: &str = env!("KERNEL_BINARY");
pub const OS_DISK: &str = env!("OS_DISK");

pub fn create_qcow_image(os_disk: &str) -> String {
    let name = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .map(char::from)
        .collect::<String>();
    let disk_image = format!("{}/{}.qcow2", env!("OUT_DIR"), name);

    let output = std::process::Command::new("qemu-img")
        .arg("create")
        .arg("-f")
        .arg("qcow2")
        .arg("-o")
        .arg(format!("backing_file={os_disk},backing_fmt=raw"))
        .arg(&disk_image)
        .output()
        .expect("failed to execute qemu-img");
    assert!(output.status.success());
    disk_image
}

pub fn run_test_kernel(kernel: &str, os_disk: &str) {
    let os_disk = create_qcow_image(os_disk);

    let mut cmd = std::process::Command::new("qemu-system-x86_64");
    cmd.arg("--no-reboot");
    cmd.arg("-d").arg("guest_errors");
    cmd.arg("-bios").arg(ovmf_prebuilt::ovmf_pure_efi());
    cmd.arg("-drive").arg(format!("format=raw,file={kernel}"));
    cmd.arg("-drive")
        .arg(format!("file={},if=ide,format=qcow2", os_disk));
    cmd.arg("-nographic");
    cmd.arg("-device")
        .arg("isa-debug-exit,iobase=0xf4,iosize=0x04");

    let output = cmd.output().expect("failed to execute qemu");
    assert_eq!(
        output.status.code(),
        Some(33),
        "test failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    ); // 33=success, 35=failed

    println!("{}", String::from_utf8_lossy(&output.stdout));
}
