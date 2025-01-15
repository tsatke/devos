static KERNEL_BINARY: &'static str = env!("KERNEL_BINARY");
static BOOTABLE_ISO: &'static str = env!("BOOTABLE_ISO");
static OVMF_CODE: &'static str = env!("OVMF_X86_64_CODE");
static OVMF_VARS: &'static str = env!("OVMF_X86_64_VARS");

fn main() {
    println!("KERNEL_BINARY: {}", KERNEL_BINARY);
    println!("BOOTABLE_ISO: {}", BOOTABLE_ISO);
    println!("OVMF_CODE: {}", OVMF_CODE);
    println!("OVMF_VARS: {}", OVMF_VARS);

    let status = std::process::Command::new("qemu-system-x86_64")
        .arg("--no-reboot")
        .arg("-serial")
        .arg("stdio")
        .arg("-monitor")
        .arg("telnet::45454,server,nowait")
        .arg("-drive")
        .arg(format!(
            "if=pflash,unit=0,format=raw,file={OVMF_CODE},readonly=on"
        ))
        .arg("-drive")
        .arg(format!("if=pflash,unit=1,format=raw,file={OVMF_VARS}"))
        .arg("-cdrom")
        .arg(BOOTABLE_ISO)
        .status()
        .unwrap();
    assert!(status.success());
}
