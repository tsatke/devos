use std::process::Stdio;

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

    cmd.stderr(Stdio::null());
    cmd.stdout(Stdio::null());
    cmd.stdin(Stdio::null());

    let mut child = cmd.spawn().unwrap();
    let exit_code = child.wait().unwrap();
    assert_eq!(exit_code.code(), Some(33)); // 33=success, 35=failed
}

#[test]
fn test_kernel_multitasking() {
    const KERNEL: &str = env!("TEST_KERNEL_MULTITASKING_PATH");
    run_test_kernel(KERNEL, OS_DISK);
}

#[test]
fn test_kernel_vfs() {
    const KERNEL: &str = env!("TEST_KERNEL_VFS_PATH");
    run_test_kernel(KERNEL, OS_DISK);
}
