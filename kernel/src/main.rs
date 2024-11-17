#![no_std]
#![no_main]

extern crate alloc;

use core::panic::PanicInfo;
use core::slice::from_raw_parts;

use bootloader_api::{entry_point, BootInfo, BootloaderConfig};
use kernel::arch::panic::handle_panic;
use kernel::driver::hpet::{hpet, HpetVolatileFieldAccess};
use kernel::process::{change_thread_priority, Priority, Process};
use kernel::{bootloader_config, kernel_init, process, serial_println};

const CONFIG: BootloaderConfig = bootloader_config();

entry_point!(kernel_main, config = &CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    let ramdisk = boot_info
        .ramdisk_addr
        .into_option()
        .map(|v| (v as *const u8, boot_info.ramdisk_len as usize))
        .map(|(addr, len)| unsafe { from_raw_parts(addr, len) });

    if ramdisk.is_some() {
        serial_println!("got a ramdisk");
    }

    kernel_init(boot_info).expect("kernel_init failed");

    let _ = Process::spawn_from_executable(
        process::current(),
        "/bin/window_server",
        Priority::Realtime,
        0.into(),
        0.into(),
    );

    let xhci = pci::devices()
        .filter(|d| {
            matches!(
                d.class(),
                PciDeviceClass::SerialBusController(SerialBusSubClass::USBController)
            ) && d.prog_if() == 0x30
        })
        .map(|d| PciStandardHeaderDevice::new(d.clone()).unwrap())
        .map(|d| XhciRegisters::try_from(d).unwrap())
        .next()
        .unwrap();

    let capabilities = xhci.capabilities.read();
    xhci.extended_capabilities().for_each(|excap| {
        if excap.read().id() == 2 {
            // we have Supported Protocol
            let spc_ptr = unsafe {
                VolatilePtr::new(excap.as_raw_ptr().cast::<SupportedProtocolCapability>())
            };
            let spc = spc_ptr.read();
            serial_println!("Supported Protocol Capability: {:#?}", spc);

            let psic = spc.psic();
            if psic > 0 {
                for i in 0..psic {
                    let psi = unsafe {
                        VolatilePtr::new(
                            spc_ptr
                                .as_raw_ptr()
                                .add(1) // end of the struct, start of the first PSI
                                .cast::<Psi>()
                                .add(usize::from(i)),
                        )
                    };
                    let psi = psi.read();
                    let exponent = psi.psie() / 3;
                    let bit_rate = usize::from(psi.psim()) * (10_usize.pow(u32::from(exponent)));

                    serial_println!(
                        "psi {} is {} operating at {} bits per second",
                        psi.psiv(),
                        match psi.lp() {
                            0 => "SuperSpeed",
                            1 => "SuperSpeedPlus",
                            2..=3 => "Reserved",
                            _ => "Unknown",
                        },
                        bit_rate,
                    );
                    serial_println!("psi: {:#?}", psi);
                }
            } else {
                serial_println!("no PSIs, default speeds apply for this protocol");
            }
        }
    });

    let num_ports = capabilities.hcsparams1.max_ports();
    for i in 1..=num_ports {
        let port = NonZeroU8::new(i).unwrap();
        let portsc = xhci.portsc(port).read();
        let current_connect_status = portsc.ccs();
        if !current_connect_status {
            continue;
        }
        serial_println!("port {} is connected", i);
        let speed = portsc.port_speed();
        serial_println!("port {} speed: {:?}", i, speed);
    }

    change_thread_priority(Priority::Low);

    panic!("kernel_main returned");
}

#[cfg(not(test))]
#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    serial_println!(
        "kernel panicked in pid={} ({}) tid={} ({}): {}",
        process::current().pid(),
        process::current().name(),
        process::current_thread().id(),
        process::current_thread().name(),
        info.message()
    );
    if let Some(location) = info.location() {
        serial_println!(
            "\tat {}:{}:{}",
            location.file(),
            location.line(),
            location.column()
        );
    }

    handle_panic(info)
}
