use clap::Parser;

extern crate clap;

// both are set in build.rs at build time
const UEFI_PATH: &str = env!("UEFI_PATH");
const BIOS_PATH: &str = env!("BIOS_PATH");

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = "The boot tool for DevOS.")]
struct Args {
    #[arg(long, help = "Boot the BIOS image rather than the UEFI one")]
    bios: bool,
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
}

fn main() {
    let args = Args::parse();

    let mut cmd = std::process::Command::new("qemu-system-x86_64");
    cmd.arg("--no-reboot");
    cmd.arg("-serial").arg("stdio");
    cmd.arg("-monitor").arg("telnet::45454,server,nowait");
    cmd.arg("-d").arg("guest_errors");
    if args.fullscreen {
        cmd.arg("-fullscreen");
    }
    if args.debug {
        cmd.arg("-gdb").arg("tcp:1234"); // long for "-s"
        cmd.arg("-S");
    }
    if args.bios {
        cmd.arg("-drive")
            .arg(format!("format=raw,file={BIOS_PATH}"));
    } else {
        cmd.arg("-bios").arg(ovmf_prebuilt::ovmf_pure_efi());
        cmd.arg("-drive")
            .arg(format!("format=raw,file={UEFI_PATH}"));
    }

    if args.verbose {
        println!("qemu command: {:?}", cmd);
    }

    let mut child = cmd.spawn().unwrap();
    child.wait().unwrap();
}
