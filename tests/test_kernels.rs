extern crate devos;

use devos::{OS_DISK, run_test_kernel};

#[test]
fn test_kernel_unittests() {
    run_test_kernel(env!("TEST_KERNEL_UNITTESTS_PATH"), OS_DISK);
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

#[test]
fn test_kernel_file_vmobject() {
    run_test_kernel(env!("TEST_KERNEL_FILE_VMOBJECT_PATH"), OS_DISK);
}
