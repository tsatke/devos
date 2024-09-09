extern crate clap;
extern crate devos;

use clap::Parser;

use devos::{create_qcow_image, KERNEL_BINARY, OS_DISK, UEFI_PATH};

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
    #[arg(long, help = "Only print the path to the UEFI image")]
    no_run: bool,
    #[arg(long, help = "Do not attach any graphical output devices")]
    headless: bool,
}

fn main() {
    let args = Args::parse();
    if args.no_run {
        println!("KERNEL_BINARY={KERNEL_BINARY}");
        println!("UEFI={UEFI_PATH}");
        println!("OS_DISK={OS_DISK}");
        return;
    }

    #[cfg(debug_assertions)]
    {
        // create an lldb debug file to make debugging easy
        let content = format!(
            r#"target create {KERNEL_BINARY}
target modules load --file {KERNEL_BINARY} --slide 0xffff800000000000
gdb-remote localhost:1234
b _start
b rust_begin_unwind"#
        );
        std::fs::write("debug.lldb", content).expect("unable to create debug file");
        println!("debug file is ready, run `lldb -s debug.lldb` to start debugging");
    }

    let mut cmd = std::process::Command::new("qemu-system-x86_64");
    cmd.arg("--no-reboot");
    cmd.arg("-serial").arg("stdio");
    cmd.arg("-monitor").arg("telnet::45454,server,nowait");
    cmd.arg("-d").arg("guest_errors");
    if args.fullscreen {
        cmd.arg("-full-screen");
    }

    #[cfg(debug_assertions)]
    cmd.arg("-s");

    if args.debug {
        cmd.arg("-S");
    }
    cmd.arg("-bios").arg(ovmf_prebuilt::ovmf_pure_efi());
    cmd.arg("-drive")
        .arg(format!("format=raw,file={UEFI_PATH}"));

    if args.headless {
        cmd.arg("-vga").arg("none");
        cmd.arg("-nographic");
    }

    // create a copy on write image
    let os_disk = create_qcow_image(OS_DISK);

    // add the os disk as hard drive
    cmd.arg("-drive")
        .arg(format!("file={os_disk},if=ide,format=qcow2"));

    if args.verbose {
        println!("qemu command: {:?}", cmd);
    }

    let mut child = cmd.spawn().unwrap();
    child.wait().unwrap();
}
