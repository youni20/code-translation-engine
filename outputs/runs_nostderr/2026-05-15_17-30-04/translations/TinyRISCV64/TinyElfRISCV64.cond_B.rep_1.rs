/*
 * TinyRISCV64 extension to add elf loading and syscall ABIs
 *
 * https://github.com/neilstephens/TinyRISCV64
 *
 * MIT License
 *
 * (c) 2025 Neil Stephens
 */

use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read, Write, Seek, SeekFrom};
use std::sync::{Arc, Mutex};
use std::time;
use std::vec::Vec;

type U64 = u64;
type I64 = i64;
type U32 = u32;

mod tinyriscv64 {
    pub trait VM {
        fn reset(&mut self);
        fn handle_semihost(&mut self);
        fn handle_ecall(&mut self);
    }
}

struct ElfVM {
    fd_streams: HashMap<U64, Arc<Mutex<Box<dyn ReadWriteSeek>>>>,
    tls_tp: U64,
    program: Vec<u8>,
    x: [U64; 32],
    pc: U64,
    halted: bool,
    max_prog_size: usize,
}

trait ReadWriteSeek: Read + Write + Seek {}

impl<T: Read + Write + Seek> ReadWriteSeek for T {}

impl ElfVM {
    pub fn new(_stack_size: usize, max_program_size: usize) -> Self {
        Self {
            fd_streams: HashMap::new(),
            tls_tp: 0,
            program: Vec::new(),
            x: [0; 32],
            pc: 0,
            halted: false,
            max_prog_size: max_program_size,
        }
    }

    pub fn program_load(&mut self, prog_filename: &str) -> io::Result<U64> {
        let (prog, entry, tp) = Self::load_elf(prog_filename, self.max_prog_size)?;
        self.tls_tp = tp;
        self.program = prog;
        self.reset();
        Ok(entry)
    }

    fn reset(&mut self) {
        self.tls_tp = 0;
        if self.tls_tp != 0 {
            self.x[4] = self.tls_tp;
        }
    }

    fn map_fd(&mut self, fd: U64, stream: Arc<Mutex<Box<dyn ReadWriteSeek>>>) {
        self.fd_streams.insert(fd, stream);
    }

    fn mem_read_str(&self, addr: U64) -> String {
        let mut s = String::new();
        let mut addr = addr;
        loop {
            let c: u8 = self.mem_load::<u8>(addr);
            if c == 0 {
                break;
            }
            s.push(c as char);
            addr += 1;
        }
        s
    }

    fn handle_semihost(&mut self) {
        let op = self.x[10];
        let arg = self.x[11];

        let argv = |n: usize| -> U64 { self.mem_load::<U64>(arg + (n as U64) * 8) };

        match op {
            0x01 => {
                let path = self.mem_read_str(argv(0));
                let mode = argv(1);

                self.x[10] = match path.as_str() {
                    ":tt" if mode < 4 => 0,
                    ":tt" if mode < 8 => 1,
                    ":tt" if mode < 12 => 2,
                    _ => U64::MAX,
                };
            }

            0x02 => {
                let fd = argv(0);
                self.fd_streams.remove(&fd);
                self.x[10] = 0;
            }

            0x03 => {
                let c = self.mem_load::<u8>(arg) as char;
                if let Some(stream) = self.fd_streams.get(&1) {
                    let _ = stream.lock().unwrap().write_all(&[c as u8]);
                }
                self.x[10] = 0;
            }

            0x04 => {
                let s = self.mem_read_str(arg);
                if let Some(stream) = self.fd_streams.get(&1) {
                    let _ = stream.lock().unwrap().write_all(s.as_bytes());
                }
                self.x[10] = 0;
            }

            0x05 => {
                let fd = argv(0);
                let buf = argv(1);
                let len = argv(2);

                if let Some(stream) = self.fd_streams.get(&fd) {
                    let mut data = Vec::new();
                    for i in 0..len {
                        data.push(self.mem_load::<u8>(buf + i) as u8);
                    }
                    if stream.lock().unwrap().write_all(&data).is_ok() {
                        self.x[10] = 0;
                    } else {
                        self.x[10] = len;
                    }
                } else {
                    self.x[10] = len;
                }
            }

            0x06 => {
                let fd = argv(0);
                let buf = argv(1);
                let len = argv(2);

                if let Some(stream) = self.fd_streams.get(&fd) {
                    let mut data = vec![0u8; len as usize];
                    match stream.lock().unwrap().read(&mut data) {
                        Ok(n) => {
                            for i in 0..n {
                                self.mem_store(buf + i as U64, data[i]);
                            }
                            self.x[10] = len - n as U64;
                        }
                        Err(_) => {
                            self.x[10] = len;
                        }
                    }
                } else {
                    self.x[10] = len;
                }
            }

            0x07 => {
                let stream = self.fd_streams.get(&0);
                if let Some(stream) = stream {
                    match stream.lock().unwrap().bytes().next() {
                        Some(Ok(byte)) => {
                            self.x[10] = byte as U64;
                        }
                        _ => {
                            self.x[10] = 0xFFFFFFFFFFFFFF04;
                        }
                    }
                } else {
                    self.x[10] = 0xFFFFFFFFFFFFFF04;
                }
            }

            0x09 => {
                let fd = argv(0);
                self.x[10] = if fd <= 2 { 1 } else { 0 };
            }

            0x0A => {
                let fd = argv(0);
                let offset = argv(1) as I64;

                if let Some(stream) = self.fd_streams.get(&fd) {
                    let _ = stream.lock().unwrap().seek(SeekFrom::Start(offset as u64));
                    if stream.lock().unwrap().stream_position().is_ok() {
                        self.x[10] = 0;
                    } else {
                        self.x[10] = U64::MAX;
                    }
                } else {
                    self.x[10] = U64::MAX;
                }
            }

            0x0C => {
                let fd = argv(0);
                if let Some(stream) = self.fd_streams.get(&fd) {
                    if let Ok(pos) = stream.lock().unwrap().stream_position() {
                        let end = stream.lock().unwrap().seek(SeekFrom::End(0)).unwrap_or(U64::MAX);
                        let _ = stream.lock().unwrap().seek(SeekFrom::Start(pos));
                        self.x[10] = end;
                    } else {
                        self.x[10] = U64::MAX;
                    }
                } else {
                    self.x[10] = U64::MAX;
                }
            }

            0x11 => {
                self.x[10] = time::SystemTime::now()
                    .duration_since(time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
            }

            0x18 | 0x30 => {
                self.x[10] = if op == 0x30 { argv(1) } else { 0 };
                self.halted = true;
            }

            _ => {
                panic!(
                    "Unsupported semihosting operation 0x{:x} at pc=0x{:x}",
                    op, self.pc - 4
                );
            }
        }
    }

    fn handle_ecall(&mut self) {
        let num = self.x[17];
        let a0 = self.x[10];
        let a1 = self.x[11];
        let a2 = self.x[12];

        match num {
            93 | 94 => {
                self.halted = true;
            }

            9 | 215 | 222 | 214 | 226 | 233 | 216 | 219 | 228 => {
                self.x[10] = U64::MAX - 38;
            }

            57 => {
                self.fd_streams.remove(&a0);
                self.x[10] = 0;
            }

            62 => {
                let offset = a1 as I64;
                if let Some(stream) = self.fd_streams.get(&a0) {
                    if stream.lock().unwrap().seek(SeekFrom::Start(offset as u64)).is_ok() {
                        self.x[10] = stream.lock().unwrap().stream_position().unwrap_or(U64::MAX);
                    } else {
                        self.x[10] = U64::MAX;
                    }
                } else {
                    self.x[10] = U64::MAX - 9;
                }
            }

            63 => {
                if let Some(stream) = self.fd_streams.get(&a0) {
                    let mut buf = vec![0u8; a2 as usize];
                    match stream.lock().unwrap().read(&mut buf) {
                        Ok(n) => {
                            for i in 0..n {
                                self.mem_store(a1 + i as U64, buf[i]);
                            }
                            self.x[10] = n as U64;
                        }
                        Err(_) => {
                            self.x[10] = U64::MAX - 5;
                        }
                    }
                } else {
                    self.x[10] = U64::MAX - 9;
                }
            }

            64 => {
                if let Some(stream) = self.fd_streams.get(&a0) {
                    let mut buf = Vec::with_capacity(a2 as usize);
                    for i in 0..a2 {
                        buf.push(self.mem_load::<u8>(a1 + i));
                    }
                    if stream.lock().unwrap().write_all(&buf).is_ok() {
                        self.x[10] = a2;
                    } else {
                        self.x[10] = U64::MAX - 5;
                    }
                } else {
                    self.x[10] = U64::MAX - 9;
                }
            }

            56 | 65 | 66 | 79 | 80 => {
                self.x[10] = U64::MAX - 38;
            }

            160 => {
                self.x[10] = U64::MAX - 38;
            }

            113 | 169 => {
                self.x[10] = U64::MAX - 22;
            }

            174 | 175 | 176 | 177 => {
                self.x[10] = 0;
            }

            96 => {
                self.x[10] = 1;
            }

            99 | 100 | 261 | 132 | 134 | 135 => {
                self.x[10] = 0;
            }

            220 | 221 => {
                self.x[10] = U64::MAX - 38;
            }

            _ => {
                panic!(
                    "ecall: unsupported syscall number {} (a0={}, a1=0x{:x}, a2={}) at pc=0x{:x}",
                    num, a0, a1, a2, self.pc - 4
                );
            }
        }
    }

    fn load_elf(filename: &str, max_size: usize) -> io::Result<(Vec<u8>, U64, U64)> {
        let mut file = File::open(filename)?;
        let mut file_data = Vec::new();
        file.read_to_end(&mut file_data)?;

        if file_data.len() < std::mem::size_of::<Elf64Ehdr>() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Too small for ELF"));
        }

        let ehdr = unsafe { &*(file_data.as_ptr() as *const Elf64Ehdr) };

        // Magic number check
        if ehdr.e_ident[0] != 0x7F || ehdr.e_ident[1] != b'E' || ehdr.e_ident[2] != b'L' || ehdr.e_ident[3] != b'F' {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid ELF magic number"));
        }

        if ehdr.e_ident[4] != 2 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid ELF class"));
        }

        if ehdr.e_ident[5] != 1 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid ELF data encoding"));
        }

        if ehdr.e_ident[6] != 1 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid ELF version"));
        }

        if ehdr.e_machine != 0xF3 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Not a RISC-V ELF"));
        }

        if ehdr.e_type != 2 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "ELF is not an executable"));
        }

        let has_interp = false;
        let has_dynamic = false;

        let mut vaddr_min = U64::MAX;
        let mut vaddr_max = 0;
        let mut tls_tp = 0;

        for i in 0..ehdr.e_phnum {
            let phdr: &Elf64Phdr = unsafe {
                &*(file_data.as_ptr().add(ehdr.e_phoff as usize + (i as usize) * std::mem::size_of::<Elf64Phdr>())
                    as *const Elf64Phdr)
            };

            match phdr.p_type {
                3 => {}
                2 => {}
                7 => {
                    tls_tp = phdr.p_vaddr;
                }
                1 => {
                    if phdr.p_filesz > phdr.p_memsz {
                        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Malformed ELF"));
                    }
                    if phdr.p_offset + phdr.p_filesz > file_data.len() as u64 {
                        return Err(io::Error::new(io::ErrorKind::InvalidInput, "PT_LOAD out of bounds"));
                    }
                    vaddr_min = vaddr_min.min(phdr.p_vaddr);
                    vaddr_max = vaddr_max.max(phdr.p_vaddr + phdr.p_memsz);
                }
                _ => {}
            }
        }

        if has_interp || has_dynamic || vaddr_min == U64::MAX {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Unsupported ELF structure"));
        }

        if vaddr_max > max_size as U64 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "ELF exceeds max program size"));
        }

        if ehdr.e_entry < vaddr_min || ehdr.e_entry >= vaddr_max {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "ELF entry point out of bounds"));
        }

        let mut prog = vec![0u8; vaddr_max as usize];

        for i in 0..ehdr.e_phnum {
            let phdr: &Elf64Phdr = unsafe {
                &*(file_data.as_ptr().add(ehdr.e_phoff as usize + (i as usize) * std::mem::size_of::<Elf64Phdr>())
                    as *const Elf64Phdr)
            };
            if phdr.p_type == 1 && phdr.p_filesz > 0 {
                prog[phdr.p_vaddr as usize
                    ..phdr.p_vaddr as usize + phdr.p_filesz as usize]
                    .copy_from_slice(&file_data[phdr.p_offset as usize
                        ..phdr.p_offset as usize + phdr.p_filesz as usize]);
            }
        }

        Ok((prog, ehdr.e_entry, tls_tp))
    }

    fn mem_load<T>(&self, _addr: U64) -> T where T: Default {
        Default::default()
    }

    fn mem_store<T>(&self, _addr: U64, _value: T) {
        unimplemented!()
    }
}

#[repr(C)]
struct Elf64Ehdr {
    e_ident: [u8; 16],
    e_type: u16,
    e_machine: u16,
    e_version: U32,
    e_entry: U64,
    e_phoff: U64,
    e_shoff: U64,
    e_flags: U32,
    e_ehsize: u16,
    e_phentsize: u16,
    e_phnum: u16,
    e_shentsize: u16,
    e_shnum: u16,
    e_shstrndx: u16,
}

#[repr(C)]
struct Elf64Phdr {
    p_type: U32,
    p_flags: U32,
    p_offset: U64,
    p_vaddr: U64,
    p_paddr: U64,
    p_filesz: U64,
    p_memsz: U64,
    p_align: U64,
}

fn main() {
}