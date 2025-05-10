use core::ffi::CStr;
use thiserror::Error;
use zerocopy::{Immutable, KnownLayout, TryFromBytes};

#[derive(Copy, Clone, Debug)]
pub struct ElfFile<'a> {
    pub(crate) source: &'a [u8],
    pub(crate) header: &'a ElfHeader,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Error)]
pub enum ElfParseError {
    #[error("could not parse elf header")]
    HeaderParseError,
    #[error("invalid magic number")]
    InvalidMagic,
    #[error("invalid e_phentsize")]
    InvalidPhEntSize,
    #[error("invalid e_shentsize")]
    InvalidShEntSize,
    #[error("unsupported os abi")]
    UnsupportedOsAbi,
    #[error("unsupported elf version")]
    UnsupportedElfVersion,
    #[error("unsupported endianness")]
    UnsupportedEndian,
}

impl<'a> ElfFile<'a> {
    pub fn try_parse(source: &'a [u8]) -> Result<Self, ElfParseError> {
        let header = ElfHeader::try_ref_from_bytes(&source[..size_of::<ElfHeader>()])
            .map_err(|_| ElfParseError::HeaderParseError)?;

        if header.ident.magic != [0x7F, 0x45, 0x4C, 0x46] {
            return Err(ElfParseError::InvalidMagic);
        }

        #[cfg(target_endian = "little")]
        const ENDIAN: u8 = 1;
        #[cfg(target_endian = "big")]
        const ENDIAN: u8 = 2;
        if header.ident.data != ENDIAN {
            return Err(ElfParseError::UnsupportedEndian);
        }

        if usize::from(header.phentsize) != size_of::<ProgramHeader>() {
            return Err(ElfParseError::InvalidPhEntSize);
        }
        if usize::from(header.shentsize) != size_of::<SectionHeader>() {
            return Err(ElfParseError::InvalidShEntSize);
        }
        if header.ident.version != 1 || header.version != 1 {
            return Err(ElfParseError::UnsupportedElfVersion);
        }
        if header.ident.os_abi != 0x00 {
            // not Sys V
            return Err(ElfParseError::UnsupportedOsAbi);
        }

        Ok(Self { source, header })
    }

    pub fn program_headers(&self) -> impl Iterator<Item = &ProgramHeader> {
        self.headers(self.header.phoff, usize::from(self.header.phnum))
    }

    pub fn program_headers_by_type(
        &self,
        typ: ProgramHeaderType,
    ) -> impl Iterator<Item = &ProgramHeader> {
        self.program_headers().filter(move |h| h.typ == typ)
    }

    pub fn section_headers(&self) -> impl Iterator<Item = &SectionHeader> {
        self.headers(self.header.shoff, usize::from(self.header.shnum))
    }

    pub fn section_headers_by_type(
        &self,
        typ: SectionHeaderType,
    ) -> impl Iterator<Item = &SectionHeader> {
        self.section_headers().filter(move |h| h.typ == typ)
    }

    #[inline(always)]
    fn headers<T: TryFromBytes + KnownLayout + Immutable + 'a>(
        &self,
        header_offset: usize,
        header_num: usize,
    ) -> impl Iterator<Item = &T> {
        let size = size_of::<T>();
        let data = &self.source[header_offset..header_offset + (usize::from(header_num) * size)];

        data.chunks_exact(size)
            .map(T::try_ref_from_bytes)
            .map(Result::unwrap)
    }

    pub fn section_data(&self, header: &SectionHeader) -> &[u8] {
        &self.source[header.offset..header.offset + header.size]
    }

    pub fn section_name(&self, header: &SectionHeader) -> Option<&str> {
        let shstrtab = self
            .section_headers()
            .nth(usize::from(self.header.shstrndx))?;
        let shstrtab_data = self.section_data(shstrtab);
        CStr::from_bytes_until_nul(&shstrtab_data[header.name as usize..])
            .ok()?
            .to_str()
            .ok()
    }

    pub fn sections_by_name(&self, name: &str) -> impl Iterator<Item = &SectionHeader> {
        self.section_headers()
            .filter(move |h| self.section_name(h) == Some(name))
    }

    pub fn program_data(&self, header: &ProgramHeader) -> &[u8] {
        &self.source[header.offset..header.offset + header.filesz]
    }
}

const _: () = {
    assert!(64 == size_of::<ElfHeader>());
};

#[derive(TryFromBytes, KnownLayout, Immutable, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct ElfHeader {
    pub ident: ElfIdent,
    pub typ: ElfType,
    pub machine: u16,
    pub version: u32,
    pub entry: usize,
    pub phoff: usize,
    pub shoff: usize,
    pub flags: u32,
    pub ehsize: u16,
    pub phentsize: u16,
    pub phnum: u16,
    pub shentsize: u16,
    pub shnum: u16,
    pub shstrndx: u16,
}

#[derive(TryFromBytes, KnownLayout, Immutable, Debug, Eq, PartialEq, Clone)]
#[repr(u16)]
pub enum ElfType {
    None = 0x00,
    Rel = 0x01,
    Exec = 0x02,
    Dyn = 0x03,
    Core = 0x04,
}

const _: () = {
    assert!(16 == size_of::<ElfIdent>());
};

#[derive(TryFromBytes, KnownLayout, Immutable, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct ElfIdent {
    pub magic: [u8; 4],
    pub class: u8,
    pub data: u8,
    pub version: u8,
    pub os_abi: u8,
    pub abi_version: u8,
    _padding: [u8; 7],
}

const _: () = {
    assert!(56 == size_of::<ProgramHeader>());
};

#[derive(TryFromBytes, KnownLayout, Immutable, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct ProgramHeader {
    pub typ: ProgramHeaderType,
    pub flags: u32,
    pub offset: usize,
    pub vaddr: usize,
    pub paddr: usize,
    pub filesz: usize,
    pub memsz: usize,
    pub align: usize,
}

#[derive(TryFromBytes, KnownLayout, Immutable, Debug, Eq, PartialEq)]
pub struct ProgramHeaderType(pub u16);

impl ProgramHeaderType {
    pub const NULL: Self = Self(0x00);
    pub const LOAD: Self = Self(0x01);
    pub const DYNAMIC: Self = Self(0x02);
    pub const INTERP: Self = Self(0x03);
    pub const NOTE: Self = Self(0x04);
    pub const SHLIB: Self = Self(0x05);
    pub const PHDR: Self = Self(0x06);
    pub const TLS: Self = Self(0x07);
}

const _: () = {
    assert!(64 == size_of::<SectionHeader>());
};

#[derive(TryFromBytes, KnownLayout, Immutable, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct SectionHeader {
    pub name: u32,
    pub typ: SectionHeaderType,
    pub flags: SectionHeaderFlags,
    pub addr: usize,
    pub offset: usize,
    pub size: usize,
    pub link: u32,
    pub info: u32,
    pub addralign: usize,
    pub entsize: usize,
}

#[derive(TryFromBytes, KnownLayout, Immutable, Debug, Eq, PartialEq, Copy, Clone)]
pub struct SectionHeaderType(pub u32);

impl SectionHeaderType {
    pub const NULL: Self = Self(0x00);
    pub const PROGBITS: Self = Self(0x01);
    pub const SYMTAB: Self = Self(0x02);
    pub const STRTAB: Self = Self(0x03);
    pub const RELA: Self = Self(0x04);
    pub const HASH: Self = Self(0x05);
    pub const DYNAMIC: Self = Self(0x06);
    pub const NOTE: Self = Self(0x07);
    pub const NOBITS: Self = Self(0x08);
    pub const REL: Self = Self(0x09);
    pub const SHLIB: Self = Self(0x0A);
    pub const DYNSYM: Self = Self(0x0B);
    pub const INITARRAY: Self = Self(0x0E);
    pub const FINIARRAY: Self = Self(0x0F);
    pub const PREINITARRAY: Self = Self(0x10);
    pub const GROUP: Self = Self(0x11);
    pub const SYMTABSHNDX: Self = Self(0x12);
    pub const NUM: Self = Self(0x13);
}

#[derive(TryFromBytes, KnownLayout, Immutable, Debug, Eq, PartialEq)]
pub struct SectionHeaderFlags(pub u32);

impl SectionHeaderFlags {
    pub const WRITE: Self = Self(0x0001);
    pub const ALLOC: Self = Self(0x0002);
    pub const EXECINSTR: Self = Self(0x0004);
    pub const MERGE: Self = Self(0x0010);
    pub const STRINGS: Self = Self(0x0020);
    pub const INFOLINK: Self = Self(0x0040);
    pub const LINKORDER: Self = Self(0x0080);
    pub const OSNONCONFORMING: Self = Self(0x0100);
    pub const GROUP: Self = Self(0x0200);
    pub const TLS: Self = Self(0x0400);

    pub fn contains(&self, other: &Self) -> bool {
        self.0 & other.0 > 0
    }
}

#[cfg(test)]
mod tests {
    use zerocopy::TryFromBytes;

    extern crate std;

    use crate::file::{ElfFile, ElfHeader, ElfIdent, ElfType};
    use std::path::*;

    #[test]
    fn test_elf_file_parse() {
        let path = Path::new(
            "/Users/tsatke/Development/devos/target/debug/build/devos-4d6851d93689b62d/out/disk/bin/sandbox",
        );
        let data = std::fs::read(path).unwrap();

        let elf_file = ElfFile::try_parse(&data).unwrap();
        elf_file
            .section_headers()
            .for_each(|header| std::println!("{:?}: {:x?}", elf_file.section_name(header), header));
    }

    #[test]
    fn test_elf_header_ref_from_bytes() {
        let data: [u8; 64] = [
            0x7f, 0x45, 0x4c, 0x46, // ELF magic
            0x02, // 64-bit
            0x01, // little-endian
            0x01, // ELF version
            0x06, // OS ABI
            0x07, // ABI Version
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // padding
            0x02, 0x00, // ET_EXEC (little endian)
            0x00, 0x00, // no specific instruction set
            0x01, 0x00, 0x00, 0x00, // ELF version 1
            0xE8, 0xE7, 0xE6, 0xE5, 0xE4, 0xE3, 0xE2, 0xE1, // entry point
            0xB8, 0xB7, 0xB6, 0xB5, 0xB4, 0xB3, 0xB2, 0xB1, // program header table offset
            0xC8, 0xC7, 0xC6, 0xC5, 0xC4, 0xC3, 0xC2, 0xC1, // section header table offset
            0xF4, 0xF3, 0xF2, 0xF1, // flags
            0x40, 0x00, // header size
            0x40, 0x00, // program header entry size
            0x22, 0x11, // num program headers
            0x40, 0x00, // section header entry size
            0x44, 0x33, // num section headers
            0x05, 0x00, // section names section header index
        ];

        let header = ElfHeader::try_ref_from_bytes(&data).unwrap();
        assert_eq!(
            header,
            &ElfHeader {
                ident: ElfIdent {
                    magic: [0x7f, 0x45, 0x4c, 0x46],
                    class: 2,
                    data: 1,
                    version: 1,
                    os_abi: 6,
                    abi_version: 7,
                    _padding: [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
                },
                typ: ElfType::Exec,
                machine: 0,
                version: 1,
                entry: 0xE1E2E3E4E5E6E7E8,
                phoff: 0xB1B2B3B4B5B6B7B8,
                shoff: 0xC1C2C3C4C5C6C7C8,
                flags: 0xF1F2F3F4,
                ehsize: 64,
                phentsize: 64,
                phnum: 0x1122,
                shentsize: 64,
                shnum: 0x3344,
                shstrndx: 5,
            }
        );
    }
}
