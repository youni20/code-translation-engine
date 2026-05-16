use std::collections::HashMap;
use std::fs::File;
use std::io::{Read};
use std::mem::size_of;
use std::sync::Arc;

const EI_CLASS: u8 = 4;
const ELFCLASS64: u8 = 2;
const EI_DATA: u8 = 5;
const ELFDATA2LSB: u8 = 1;
const EI_VERSION: u8 = 6;
const EV_CURRENT: u8 = 1;
const EM_RISCV: u16 = 0xF3;
const ET_EXEC: u16 = 2;
const PT_LOAD: u32 = 1;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Default)]
pub struct VM {
    x: [u64; 32],
    halted: bool,
}

pub struct ElfVM {
    x: [u64; 32],
    halted: bool,
    fd_streams: HashMap<u64, Arc<dyn std::io::Write + Send + Sync>>,
    tls_tp: u64,
    max_prog_size: usize,
}

impl ElfVM {
    pub fn new(_stack_size: usize, max_program_size: usize) -> Self {
        ElfVM {
            x: [0; 32],
            halted: false,
            fd_streams: HashMap::new(),
            tls_tp: 0,
            max_prog_size: max_program_size,
        }
    }

    pub fn program_load(&mut self, prog_filename: &str) -> Result<u64> {
        let (_prog, entry, tp) = Self::load_elf(prog_filename, self.max_prog_size)?;
        self.tls_tp = tp;
        self.reset();
        Ok(entry)
    }

    pub fn reset(&mut self) {
        self.halted = false;
        if self.tls_tp != 0 {
            self.x[4] = self.tls_tp;
        }
    }

    pub fn map_fd(&mut self, fd: u64, stream: Arc<dyn std::io::Write + Send + Sync>) {
        self.fd_streams.insert(fd, stream);
    }

    fn load_elf(filename: &str, max_size: usize) -> Result<(Vec<u8>, u64, u64)> {
        let mut file = File::open(filename)?;
        let file_size = file.metadata()?.len() as usize;

        if file_size < size_of::<Elf64Ehdr>() {
            return Err("File too small to be a valid ELF64 binary".into());
        }

        let mut file_data = vec![0u8; file_size];
        file.read_exact(&mut file_data)?;

        let ehdr: Elf64Ehdr = unsafe { std::ptr::read(file_data.as_ptr() as *const _) };

        if &file_data[0..4] != &[0x7f, b'E', b'L', b'F'] {
            return Err("Not an ELF file (bad magic number)".into());
        }

        if file_data[EI_CLASS as usize] != ELFCLASS64 {
            return Err(format!(
                "ELF is {}; recompile with riscv64-unknown-elf-gcc (EI_CLASS={})",
                if file_data[EI_CLASS as usize] == 1 {
                    "32-bit"
                } else {
                    "unknown class"
                },
                file_data[EI_CLASS as usize]
            )
            .into());
        }

        if file_data[EI_DATA as usize] != ELFDATA2LSB {
            return Err(format!(
                "ELF is big-endian; ensure the target triple is riscv64 (EI_DATA={})",
                file_data[EI_DATA as usize]
            )
            .into());
        }

        if file_data[EI_VERSION as usize] != EV_CURRENT {
            return Err(format!("Unknown ELF version (EI_VERSION={})", file_data[EI_VERSION as usize]).into());
        }

        if ehdr.e_machine != EM_RISCV {
            return Err(format!(
                "Not a RISC-V ELF (e_machine=0x{:x}); recompile targeting riscv64",
                ehdr.e_machine
            )
            .into());
        }

        if ehdr.e_type != ET_EXEC {
            return Err(format!("ELF is not an executable (e_type={}); expected ET_EXEC (2)", ehdr.e_type).into());
        }

        if ehdr.e_phoff == 0 || ehdr.e_phnum == 0 {
            return Err("ELF has no program headers; link as a static executable, not a relocatable object (.o)".into());
        }

        if ehdr.e_phentsize < size_of::<Elf64Phdr>() as u16 {
            return Err(format!(
                "ELF program header entry size too small (e_phentsize={}; expected >= {})",
                ehdr.e_phentsize,
                size_of::<Elf64Phdr>()
            )
            .into());
        }

        let phtab_end = ehdr.e_phoff as usize + ehdr.e_phnum as usize * ehdr.e_phentsize as usize;
        if phtab_end > file_size {
            return Err("ELF program header table extends beyond end of file".into());
        }

        // ----- first pass: validate segments and compute address span ----------
        let mut vaddr_min = std::u64::MAX;
        let mut vaddr_max = 0;

        for i in 0..ehdr.e_phnum {
            let phdr: Elf64Phdr = unsafe {
                std::ptr::read(
                    file_data.as_ptr().add(ehdr.e_phoff as usize + i as usize * ehdr.e_phentsize as usize) as *const _,
                )
            };

            match phdr.p_type {
                PT_LOAD => {
                    if phdr.p_filesz > phdr.p_memsz {
                        return Err(format!(
                            "ELF PT_LOAD segment[{}]: p_filesz ({}) > p_memsz ({}) — malformed ELF",
                            i, phdr.p_filesz, phdr.p_memsz
                        )
                        .into());
                    }

                    if phdr.p_offset + phdr.p_filesz > file_data.len() as u64 {
                        return Err(format!(
                            "ELF PT_LOAD segment[{}] file data extends beyond end of file",
                            i
                        )
                        .into());
                    }

                    vaddr_min = std::cmp::min(vaddr_min, phdr.p_vaddr);
                    vaddr_max = std::cmp::max(vaddr_max, phdr.p_vaddr + phdr.p_memsz);
                }
                _ => {}
            }
        }

        if vaddr_min == std::u64::MAX {
            return Err("ELF has no loadable (PT_LOAD) segments — nothing to execute".into());
        }

        if vaddr_max > max_size as u64 {
            return Err(format!(
                "ELF virtual address span [0x{:x}, 0x{:x}) requires {} bytes which exceeds max_program_size={}; ",
                vaddr_min, vaddr_max, vaddr_max, max_size
            )
            .into());
        }

        if ehdr.e_entry < vaddr_min || ehdr.e_entry >= vaddr_max {
            return Err(format!(
                "ELF entry point 0x{:x} lies outside the loaded virtual address range [0x{:x}, 0x{:x}); ",
                ehdr.e_entry, vaddr_min, vaddr_max
            )
            .into());
        }

        let mut prog = vec![0_u8; vaddr_max as usize];

        for i in 0..ehdr.e_phnum {
            let phdr: Elf64Phdr = unsafe {
                std::ptr::read(
                    file_data.as_ptr().add(ehdr.e_phoff as usize + i as usize * ehdr.e_phentsize as usize) as *const _,
                )
            };

            if phdr.p_type == PT_LOAD && phdr.p_filesz != 0 {
                prog[phdr.p_vaddr as usize..(phdr.p_vaddr + phdr.p_filesz) as usize]
                    .copy_from_slice(&file_data[(phdr.p_offset as usize)..(phdr.p_offset + phdr.p_filesz) as usize]);
            }
        }

        Ok((prog, ehdr.e_entry, 0)) // Corrected this line by replacing tls_tp with 0
    }
}

#[repr(C)]
struct Elf64Ehdr {
    e_ident: [u8; 16],
    e_type: u16,
    e_machine: u16,
    e_version: u32,
    e_entry: u64,
    e_phoff: u64,
    e_shoff: u64,
    e_flags: u32,
    e_ehsize: u16,
    e_phentsize: u16,
    e_phnum: u16,
    e_shentsize: u16,
    e_shnum: u16,
    e_shstrndx: u16,
}

#[repr(C)]
struct Elf64Phdr {
    p_type: u32,
    p_flags: u32,
    p_offset: u64,
    p_vaddr: u64,
    p_paddr: u64,
    p_filesz: u64,
    p_memsz: u64,
    p_align: u64,
}

fn main() {}