#![no_std]
#![no_main]
extern crate alloc;

use alloc::string::ToString;
use alloc::vec;
use core::arch::asm;
use core::panic::PanicInfo;
use core::slice::{from_raw_parts, from_raw_parts_mut};
use elf::endian::{AnyEndian, LittleEndian};
use elf::file::Class;
use elf::symbol::SymbolTable;
use elf::ElfBytes;
use jiff::Timestamp;
use kernel::driver::block::block_devices;
use kernel::driver::vga::{vga_devices, VgaDevice};
use kernel::limine::{BASE_REVISION, KERNEL_FILE_REQUEST};
use kernel::mem::address_space::AddressSpace;
use kernel::mem::virt::VirtualMemoryHigherHalf;
use kernel::time::TimestampExt;
use kernel::{mcore, U64Ext};
use log::{error, info};
use rustc_demangle::demangle;
use x86_64::instructions::hlt;
use x86_64::structures::paging::{PageSize, PageTableFlags, Size4KiB};
use x86_64::VirtAddr;

#[unsafe(export_name = "kernel_main")]
unsafe extern "C" fn main() -> ! {
    assert!(BASE_REVISION.is_supported());

    kernel::init();

    let blk1 = block_devices().read().get(0).cloned().unwrap();
    let mut guard = blk1.write();
    let mut data = vec![0_u8; 512];
    let start = Timestamp::now();
    guard.read(0, &mut data);
    let end = Timestamp::now();
    info!(
        "read (took {:?}): {}",
        end - start,
        str::from_utf8(&data).unwrap()
    );

    mcore::turn_idle()
}

#[panic_handler]
fn rust_panic(info: &PanicInfo) -> ! {
    handle_panic(info);
    loop {
        hlt();
    }
}

fn handle_panic(info: &PanicInfo) {
    let location = info.location().unwrap();
    error!(
        "kernel panicked at {}:{}:{}:",
        location.file(),
        location.line(),
        location.column(),
    );
    error!("{}", info.message());
    error!("stack backtrace:");
    stacktrace(|frame, addr, sym| {
        error!("\t{frame:2}: {addr:p} @ {sym}");
    });
}

fn stacktrace<F>(f: F)
where
    F: Fn(usize, VirtAddr, &str),
{
    let kernel_file = KERNEL_FILE_REQUEST.get_response().unwrap();
    let file_addr = VirtAddr::from_ptr(kernel_file.file().addr());
    let file_size = kernel_file.file().size().into_usize();
    let file_slice = unsafe { from_raw_parts(file_addr.as_mut_ptr::<u8>(), file_size) };
    let file = ElfBytes::<AnyEndian>::minimal_parse(file_slice).unwrap();
    let hdr = file
        .section_header_by_name(".symtab")
        .unwrap()
        .expect("should have .symtab");
    let symtab_data = file.section_data(&hdr).unwrap();
    let symtab = SymbolTable::new(LittleEndian, Class::ELF64, symtab_data.0);
    let strtab_data = file
        .section_header_by_name(".strtab")
        .unwrap()
        .expect("should have .strtab");
    let strtab = file.section_data_as_strtab(&strtab_data).unwrap();

    let my_rbp: *const u64;
    unsafe {
        asm!(
        "mov {}, rbp",
        out(reg) my_rbp,
        );
    }

    let mut rbp = my_rbp;
    let mut count = 0;
    while !rbp.is_null() {
        let next_rbp = unsafe { *rbp };
        let instruction_pointer = unsafe { *(rbp.add(1)) };
        rbp = next_rbp as *const u64;

        let sym = symtab
            .iter()
            .find(|v| (v.st_value..v.st_value + v.st_size).contains(&instruction_pointer))
            .map(|s| strtab.get(s.st_name as usize).unwrap())
            .map(demangle);
        if let Some(sym) = sym {
            let sym = sym.to_string();
            f(count, VirtAddr::new(instruction_pointer), &sym);
        } else {
            f(count, VirtAddr::new(instruction_pointer), "<unknown>");
        }
        count += 1;
    }
}
