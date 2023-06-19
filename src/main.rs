use clap::Parser;
use std::process::ExitStatus;

extern crate clap;

// those are set in build.rs at build time
const UEFI_PATH: &str = env!("UEFI_PATH");
const UEFI_TEST_PATH: &str = env!("UEFI_TEST_PATH");
const BIOS_PATH: &str = env!("BIOS_PATH");

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
    #[arg(long, hide = true, default_value = "false")]
    headless: bool, // used only for tests, thus hidden
}

fn main() {
    let args = Args::parse();
    if args.no_run {
        println!("UEFI={}", UEFI_PATH);
        println!("UEFI_TEST={}", UEFI_TEST_PATH);
        println!("BIOS={}", BIOS_PATH);
        return;
    }

    if cfg!(feature = "bios") {
        run_kernel_image(args, BIOS_PATH);
    } else {
        run_kernel_image(args, UEFI_PATH);
    }
}

fn run_kernel_image(args: Args, kernel: &str) -> ExitStatus {
    let mut cmd = std::process::Command::new("qemu-system-x86_64");
    cmd.arg("--no-reboot");
    cmd.arg("-serial").arg("stdio");
    cmd.arg("-monitor").arg("telnet::45454,server,nowait");
    cmd.arg("-device")
        .arg("isa-debug-exit,iobase=0xf4,iosize=0x04");
    cmd.arg("-d").arg("guest_errors");
    if args.headless {
        cmd.arg("-nographic");
    }
    if args.fullscreen {
        cmd.arg("-fullscreen");
    }
    if args.debug {
        cmd.arg("-s");
        cmd.arg("-S");
    }
    if cfg!(feature = "bios") {
        cmd.arg("-drive").arg(format!("format=raw,file={kernel}"));
    } else {
        cmd.arg("-bios").arg(ovmf_prebuilt::ovmf_pure_efi());
        cmd.arg("-drive").arg(format!("format=raw,file={kernel}"));
    }

    if args.verbose {
        println!("qemu command: {:?}", cmd);
    }

    let mut child = cmd.spawn().unwrap();
    child.wait().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_all_kernel_unittests() {
        let status = run_kernel_image(
            Args {
                verbose: false,
                fullscreen: false,
                debug: false,
                no_run: false,
                headless: true,
            },
            UEFI_TEST_PATH,
        );
        assert_eq!(0x10, status.code().unwrap() >> 1); // FIXME: why is this being shifted?
    }
}
