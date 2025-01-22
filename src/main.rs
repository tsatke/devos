use clap::Parser;

static KERNEL_BINARY: &'static str = env!("KERNEL_BINARY");
static BOOTABLE_ISO: &'static str = env!("BOOTABLE_ISO");
static OVMF_CODE: &'static str = env!("OVMF_X86_64_CODE");
static OVMF_VARS: &'static str = env!("OVMF_X86_64_VARS");

#[derive(Parser)]
struct Args {
    #[arg(
        long,
        help = "Start QEMU with a GDB server listening on localhost:1234"
    )]
    debug: bool,
}

fn main() {
    println!("KERNEL_BINARY: {}", KERNEL_BINARY);
    println!("BOOTABLE_ISO: {}", BOOTABLE_ISO);
    println!("OVMF_CODE: {}", OVMF_CODE);
    println!("OVMF_VARS: {}", OVMF_VARS);

    let args = Args::parse();

    #[cfg(debug_assertions)]
    {
        // create an lldb debug file to make debugging easy
        let content = format!(
            r#"target create {KERNEL_BINARY}

# If the kernel is a position independent executable (PIE), you need to set the slide as the offset
# at which the kernel is being loaded. For static executables, the slide is 0, in which case
# we can omit this whole line.
#
# target modules load --file {KERNEL_BINARY} --slide 0xffffffff80000000

gdb-remote localhost:1234
b kernel_main
b rust_begin_unwind
continue"#
        );
        std::fs::write("debug.lldb", content).expect("unable to create debug file");
        println!("debug file is ready, run `lldb -s debug.lldb` to start debugging");
    }

    let mut cmd = std::process::Command::new("qemu-system-x86_64");
    cmd.arg("--no-reboot");
    cmd.arg("-serial");
    cmd.arg("stdio");
    cmd.arg("-monitor");
    cmd.arg("telnet::45454,server,nowait");
    cmd.arg("-s");
    if args.debug {
        cmd.arg("-S");
    }
    cmd.arg("-drive");
    cmd.arg(format!(
        "if=pflash,unit=0,format=raw,file={OVMF_CODE},readonly=on"
    ));
    cmd.arg("-drive");
    cmd.arg(format!("if=pflash,unit=1,format=raw,file={OVMF_VARS}"));
    cmd.arg("-cdrom");
    cmd.arg(BOOTABLE_ISO);

    let status = cmd.status().unwrap();
    assert!(status.success());
}
