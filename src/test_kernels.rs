use OS_DISK;

fn run_test_kernel(kernel: &str, os_disk: &str) {
    let mut cmd = std::process::Command::new("qemu-system-x86_64");
    cmd.arg("--no-reboot");
    cmd.arg("-d").arg("guest_errors");
    cmd.arg("-bios").arg(ovmf_prebuilt::ovmf_pure_efi());
    cmd.arg("-drive").arg(format!("format=raw,file={kernel}"));
    cmd.arg("-drive")
        .arg(format!("file={},if=ide,format=raw", os_disk));
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
}

#[test]
fn test_kernel_multitasking() {
    run_test_kernel(env!("TEST_KERNEL_MULTITASKING_PATH"), OS_DISK);
}

#[test]
fn test_kernel_vfs() {
    run_test_kernel(env!("TEST_KERNEL_VFS_PATH"), OS_DISK);
}

#[test]
fn test_kernel_vmobject() {
    run_test_kernel(env!("TEST_KERNEL_VMOBJECT_PATH"), OS_DISK);
}
