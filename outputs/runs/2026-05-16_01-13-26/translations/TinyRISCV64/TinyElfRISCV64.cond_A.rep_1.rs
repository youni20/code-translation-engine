use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read};
use std::sync::{Arc, Mutex};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
type ElfProgram = (Vec<u8>, u64, u64);

trait IoStream: io::Read + io::Write + Send {}
impl<T: io::Read + io::Write + Send> IoStream for T {}

#[derive(Default)]
struct ElfVM {
    fd_streams: HashMap<u64, Arc<Mutex<dyn IoStream>>>,
    tls_tp: u64,
    program: Vec<u8>,
    max_program_size: usize,
    halted: bool,
}

impl ElfVM {
    pub fn new(stack_size: usize, max_program_size: usize) -> Self {
        ElfVM {
            max_program_size,
            ..Default::default()
        }
    }

    pub fn program_load(&mut self, prog_filename: &str) -> Result<u64> {
        let (prog, entry, tp) = ElfVM::load_elf(prog_filename, self.max_program_size)?;
        self.tls_tp = tp;
        self.program = prog;
        self.reset();
        Ok(entry)
    }

    pub fn reset(&mut self) {
        self.halted = false;
    }

    pub fn map_fd(&mut self, fd: u64, stream: Arc<Mutex<dyn IoStream>>) {
        self.fd_streams.insert(fd, stream);
    }

    fn mem_read_str(&self, _addr: u64) -> String {
        String::new()
    }

    fn handle_semihost(&mut self) {
        let op = 0;
        let arg = 0;
        
        let argv = |n: usize| -> u64 { 0 };
        
        match op {
            0x01 => {
                let path = self.mem_read_str(argv(0));
                let mode = argv(1);

                if path == ":tt" {
                    let fd = match mode {
                        0..=3 => 0,
                        4..=7 => 1,
                        8..=11 => 2,
                        _ => u64::MAX,
                    };
                } else {
                }
            }
            0x02 => {
                let fd = argv(0);
                self.fd_streams.remove(&fd);
            }
            _ => {}
        }
    }

    fn handle_ecall(&mut self) {
        let num = 0;
        let a0 = 0;
        let a1 = 0;
        let a2 = 0;

        match num {
            93 | 94 => {
                self.halted = true;
            }
            57 => {
                self.fd_streams.remove(&a0);
            }
            _ => {}
        }
    }

    fn load_elf(filename: &str, max_size: usize) -> Result<ElfProgram> {
        let mut file = File::open(filename)?;
        let file_size = file.metadata()?.len() as usize;
        if file_size < std::mem::size_of::<Elf64Ehdr>() {
            return Err("File too small to be a valid ELF64 binary".into());
        }

        let mut file_data = vec![0u8; file_size];
        file.read_exact(&mut file_data)?;

        let ehdr: Elf64Ehdr = unsafe { std::ptr::read(file_data.as_ptr() as *const _) };

        if !ehdr.is_valid() {
            return Err("Invalid ELF header".into());
        }

        let mut vaddr_min = u64::MAX;
        let mut vaddr_max = 0;
        let mut tls_tp = 0;

        for i in 0..ehdr.e_phnum {
            let phdr: Elf64Phdr = unsafe {
                std::ptr::read(file_data.as_ptr().add(ehdr.e_phoff as usize + i as usize * ehdr.e_phentsize as usize) as *const _)
            };
            match phdr.p_type {
                1 => {
                    vaddr_min = vaddr_min.min(phdr.p_vaddr);
                    vaddr_max = vaddr_max.max(phdr.p_vaddr + phdr.p_memsz);
                }
                7 => {
                    tls_tp = phdr.p_vaddr;
                }
                _ => {}
            }
        }

        if vaddr_max > max_size as u64 {
            return Err("ELF virtual address span exceeds max_program_size".into());
        }

        let mut prog = vec![0u8; vaddr_max as usize];
        for i in 0..ehdr.e_phnum {
            let phdr: Elf64Phdr = unsafe {
                std::ptr::read(file_data.as_ptr().add(ehdr.e_phoff as usize + i as usize * ehdr.e_phentsize as usize) as *const _)
            };
            if phdr.p_type == 1 && phdr.p_filesz > 0 {
                prog[phdr.p_vaddr as usize..(phdr.p_vaddr + phdr.p_filesz) as usize]
                    .copy_from_slice(&file_data[phdr.p_offset as usize..(phdr.p_offset + phdr.p_filesz) as usize]);
            }
        }

        Ok((prog, ehdr.e_entry, tls_tp))
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

impl Elf64Ehdr {
    fn is_valid(&self) -> bool {
        self.e_ident[0] == 0x7f
            && self.e_ident[1] == b'E'
            && self.e_ident[2] == b'L'
            && self.e_ident[3] == b'F'
            && self.e_ident[4] == 2
            && self.e_ident[5] == 1
            && self.e_ident[6] == 1
            && self.e_machine == 0xF3
            && [2, 3].contains(&self.e_type)
    }
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

fn main() {
    // Main function as entry point for the program.
}