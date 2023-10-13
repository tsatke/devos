use clap::Parser;
use std::fs;

extern crate clap;

// both are set in build.rs at build time
const UEFI_PATH: &str = env!("UEFI_PATH");
const BIOS_PATH: &str = env!("BIOS_PATH");
const KERNEL_BINARY: &str = env!("KERNEL_BINARY");
const OS_DISK: &str = env!("OS_DISK");

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = "The boot tool for DevOS.")]
struct Args {
    #[arg(
        short,
        long,
        help = "Print more information that may be helpful for debugging"
    )]
    verbose: bool,
    #[arg(long, help = "Boot QEMU in fullscreen mode")]
    fullscreen: bool,
    #[arg(
        long,
        help = "Start a gdb server on tcp:1234 and wait until a client has connected"
    )]
    debug: bool,
    #[arg(long, help = "Only print the paths to the UEFI and BIOS images")]
    no_run: bool,
}

fn main() {
    let args = Args::parse();
    if args.no_run {
        println!("KERNEL_BINARY={}", KERNEL_BINARY);
        println!("UEFI={}", UEFI_PATH);
        println!("BIOS={}", BIOS_PATH);
        return;
    }

    if args.debug {
        // create an lldb debug file to make debugging easy
        let content = format!(
            r#"target create {KERNEL_BINARY}
target modules load --file {KERNEL_BINARY} --slide 0x8000000000
gdb-remote localhost:1234
b _start
c"#
        );
        fs::write("debug.lldb", content).expect("unable to create debug file");
        println!("debug file is ready, run `lldb -s debug.lldb` to start debugging");
    }

    let mut cmd = std::process::Command::new("qemu-system-x86_64");
    cmd.arg("--no-reboot");
    cmd.arg("-serial").arg("stdio");
    cmd.arg("-monitor").arg("telnet::45454,server,nowait");
    cmd.arg("-d").arg("guest_errors");
    if args.fullscreen {
        cmd.arg("-fullscreen");
    }
    if args.debug {
        cmd.arg("-s");
        cmd.arg("-S");
    }
    if cfg!(feature = "bios") {
        cmd.arg("-drive")
            .arg(format!("format=raw,file={BIOS_PATH}"));
    } else {
        cmd.arg("-bios").arg(ovmf_prebuilt::ovmf_pure_efi());
        cmd.arg("-drive")
            .arg(format!("format=raw,file={UEFI_PATH}"));
    }

    // add the os disk as hard drive
    cmd.arg("-drive")
        .arg(format!("file={},if=ide,format=raw", OS_DISK));

    if args.verbose {
        println!("qemu command: {:?}", cmd);
    }

    let mut child = cmd.spawn().unwrap();
    child.wait().unwrap();
}
