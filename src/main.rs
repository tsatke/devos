use clap::Parser;
use qemu_api::args::{Drive, Format, LogItem};
use qemu_api::chardev::QemuCharDevice;
use qemu_api::{Qemu, QemuSystem, X86_64};
use std::path::PathBuf;
use std::process::{Command, ExitStatus};

extern crate clap;
extern crate qemu_api;

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
}

fn main() {
    let args = Args::parse();
    if args.no_run {
        println!("UEFI={}", UEFI_PATH);
        println!("UEFI_TEST={}", UEFI_TEST_PATH);
        println!("BIOS={}", BIOS_PATH);
        return;
    }

    let modifier = |qemu: &mut Qemu<X86_64>| {
        if args.fullscreen {
            qemu.fullscreen();
        }
        if args.debug {
            qemu.gdb(&"tcp::1234");
            qemu.freeze_on_startup();
        }

        if args.verbose {
            println!("qemu command: {:?}", qemu);
        }
    };
    if cfg!(feature = "bios") {
        run_kernel_image(BIOS_PATH, modifier);
    } else {
        run_kernel_image(UEFI_PATH, modifier);
    }
}

fn default_qemu<S>() -> Qemu<S>
where
    S: QemuSystem + Default,
{
    let mut qemu = Qemu::<S>::new();
    qemu.no_reboot();
    qemu.serial(QemuCharDevice::Stdio);
    qemu.log_items([LogItem::GuestErrors]);
    qemu.other("-monitor").other("telnet::45454,server,nowait");

    if !cfg!(feature = "bios") {
        qemu.bios(ovmf_prebuilt::ovmf_pure_efi());
    }
    qemu
}

fn run_kernel_image<F, S>(kernel: &str, modifier: F) -> ExitStatus
where
    F: Fn(&mut Qemu<S>),
    S: QemuSystem + Default,
{
    let mut qemu = default_qemu::<S>();
    qemu.drive(Drive {
        file: PathBuf::from(kernel),
        format: Some(Format::Raw),
        ..Default::default()
    });

    modifier(&mut qemu);

    let mut cmd = Command::from(qemu);

    let mut child = cmd.spawn().unwrap();
    child.wait().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_all_kernel_unittests() {
        let add_exit_device = |qemu: &mut Qemu<X86_64>| {
            qemu.other("-nographic");
            qemu.other("-device")
                .other("isa-debug-exit,iobase=0xf4,iosize=0x04");
        };
        let status = run_kernel_image(UEFI_TEST_PATH, add_exit_device);
        assert_eq!(
            0,
            status.code().unwrap() >> 1,
            "return code indicates test failures"
        ); // FIXME: why is this being shifted?
    }
}
