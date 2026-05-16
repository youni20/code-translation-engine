use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};
use std::cmp::{min, max};

type U64 = u64;
type U8 = u8;
type I64 = i64;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

struct ElfVM {
    fd_streams: HashMap<U64, Rc<RefCell<dyn ReadWriteSeek>>>,
    tls_tp: U64,
    x: [U64; 32],
    halted: bool,
    pc: U64,
    max_prog_size: usize
}

trait ReadWriteSeek: Read + Write + Seek {}

impl<T: Read + Write + Seek> ReadWriteSeek for T {}

impl ElfVM {
    pub fn new(_stack_size: usize, max_program_size: usize) -> Self {
        ElfVM {
            fd_streams: HashMap::new(),
            tls_tp: 0,
            x: [0; 32],
            halted: false,
            pc: 0,
            max_prog_size: max_program_size
        }
    }

    pub fn program_load(&mut self, prog_filename: &str) -> Result<U64> {
        let (_prog, entry, tp) = ElfVM::load_elf(prog_filename, self.max_prog_size)?;
        self.tls_tp = tp;
        self.reset();
        Ok(entry)
    }

    pub fn reset(&mut self) {
        // VM reset logic
        if self.tls_tp != 0 {
            self.x[4] = self.tls_tp;
        }
    }

    pub fn map_fd(&mut self, fd: U64, stream: Rc<RefCell<dyn ReadWriteSeek>>) {
        self.fd_streams.insert(fd, stream);
    }

    fn mem_read_str(&self, addr: U64) -> String {
        let mut s = String::new();
        let mut addr = addr;
        loop {
            let c = self.mem_load::<U8>(addr) as char;
            addr += 1;
            if c == '\0' {
                break;
            }
            s.push(c);
        }
        s
    }

    fn handle_semihost(&mut self) {
        let op = self.x[10];
        let arg = self.x[11];
        
        let argv = |n: usize| self.mem_load::<U64>(arg + n as U64 * 8);

        match op {
            0x01 => {
                let path = self.mem_read_str(argv(0));
                let mode = argv(1);

                self.x[10] = match path.as_str() {
                    ":tt" => {
                        if mode < 4 {
                            0
                        } else if mode < 8 {
                            1
                        } else if mode < 12 {
                            2
                        } else {
                            !0
                        }
                    },
                    _ => !0,
                };
            },
            0x02 => {
                let fd = argv(0);
                self.fd_streams.remove(&fd);
                self.x[10] = 0;
            },
            0x03 => {
                let c = self.mem_load::<U8>(arg) as char;
                if let Some(stream) = self.fd_streams.get(&1) {
                    stream.borrow_mut().write_all(&[c as u8]).ok();
                }
                self.x[10] = 0;
            },
            0x04 => {
                let s = self.mem_read_str(arg);
                if let Some(stream) = self.fd_streams.get(&1) {
                    stream.borrow_mut().write_all(s.as_bytes()).ok();
                }
                self.x[10] = 0;
            },
            0x05 => {
                let fd = argv(0);
                let buf = argv(1);
                let mut len = argv(2);
                if let Some(stream) = self.fd_streams.get(&fd) {
                    let mut i = 0;
                    while len > 0 {
                        let byte = self.mem_load::<U8>(buf + i);
                        stream.borrow_mut().write_all(&[byte]).ok();
                        len -= 1;
                        i += 1;
                    }
                }
                self.x[10] = len;
            },
            0x06 => {
                let fd = argv(0);
                let buf = argv(1);
                let len = argv(2);

                if let Some(stream) = self.fd_streams.get(&fd) {
                    let mut tmp = vec![0; len as usize];
                    let n = stream.borrow_mut().read(&mut tmp).unwrap_or(0);
                    for i in 0..n {
                        self.mem_store(buf + i as U64, tmp[i]);
                    }
                    self.x[10] = len - n as U64;
                }
            },
            0x07 => {
                if let Some(stream) = self.fd_streams.get(&0) {
                    let mut buffer = [0];
                    let result = stream.borrow_mut().read(&mut buffer);
                    match result {
                        Ok(1) => self.x[10] = buffer[0] as U64,
                        _ => self.x[10] = 0xFFFFFFFFFFFFFF04,
                    }
                }
            },
            0x09 => {
                let fd = argv(0);
                self.x[10] = if fd <= 2 { 1 } else { 0 };
            },
            0x0a => {
                let fd = argv(0);
                let offset = argv(1) as I64;
                if let Some(stream) = self.fd_streams.get(&fd) {
                    stream.borrow_mut().seek(SeekFrom::Start(offset as u64)).ok();
                    self.x[10] = 0;
                }
            },
            0x0c => {
                let fd = argv(0);
                if let Some(stream) = self.fd_streams.get(&fd) {
                    let cur = stream.borrow_mut().stream_position().unwrap();
                    stream.borrow_mut().seek(SeekFrom::End(0)).ok();
                    let len = stream.borrow_mut().stream_position().unwrap();
                    stream.borrow_mut().seek(SeekFrom::Start(cur)).ok();
                    self.x[10] = len;
                }
            },
            0x11 => {
                self.x[10] = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
            },
            0x18 => {
                self.halted = true;
                self.x[10] = 0;
            },
            0x30 => {
                let argv_1 = argv(1);
                self.halted = true;
                self.x[10] = argv_1;
            },
            _ => panic!("Unsupported semihosting operation 0x{:x} at pc=0x{:x}", op, self.pc - 4),
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
            },
            9 | 215 | 222 | 214 | 226 | 233 | 216 | 219 | 228 => {
                self.x[10] = !38;
            },
            57 => {
                self.fd_streams.remove(&a0);
                self.x[10] = 0;
            },
            62 => {
                if let Some(stream) = self.fd_streams.get(&a0) {
                    let dirs = [SeekFrom::Start(0), SeekFrom::Current(0), SeekFrom::End(0)];
                    if a2 > 2 {
                        self.x[10] = !22;
                        return;
                    }
                    stream.borrow_mut().seek(dirs[a2 as usize]).ok();
                    self.x[10] = stream.borrow_mut().stream_position().unwrap_or(!29) as U64;
                } else {
                    self.x[10] = !9;
                }
            },
            63 => {
                if let Some(stream) = self.fd_streams.get(&a0) {
                    let mut buf = vec![0; a2 as usize];
                    let n = stream.borrow_mut().read(&mut buf).unwrap_or(0);
                    for i in 0..n {
                        self.mem_store(a1 + i as U64, buf[i]);
                    }
                    self.x[10] = n as U64;
                } else {
                    self.x[10] = !9;
                }
            },
            64 => {
                if let Some(stream) = self.fd_streams.get(&a0) {
                    let mut buf = vec![0u8; a2 as usize];
                    for i in 0..a2 {
                        buf[i as usize] = self.mem_load::<U8>(a1 + i);
                    }
                    stream.borrow_mut().write_all(&buf).ok();
                    self.x[10] = a2;
                } else {
                    self.x[10] = !9;
                }
            },
            56 | 65 | 66 | 79 | 80 => {
                self.x[10] = !38;
            },
            160 => {
                self.x[10] = !38;
            },
            113 | 169 => {
                self.x[10] = !22;
            },
            174 | 175 | 176 | 177 => {
                self.x[10] = 0;
            },
            96 => {
                self.x[10] = 1;
            },
            99 | 100 | 261 | 132 | 134 | 135 => {
                self.x[10] = 0;
            },
            220 | 221 => {
                self.x[10] = !38;
            },
            _ => panic!("ecall: unsupported syscall number {} (a0={}, a1=0x{:x}, a2={}) at pc=0x{:x}", num, a0, a1, a2, self.pc - 4),
        }
    }

    fn mem_load<T>(&self, _addr: U64) -> T {
        // Implement this function to load a value from memory.
        unimplemented!()
    }

    fn mem_store<T>(&self, _addr: U64, _value: T) {
        // Implement this function to store a value into memory.
        unimplemented!()
    }

    fn load_elf(filename: &str, max_size: usize) -> Result<(Vec<U8>, U64, U64)> {
        let mut file = File::open(filename)?;

        // Read entire file
        let file_size = file.metadata()?.len() as usize;
        if file_size < std::mem::size_of::<Elf64Ehdr>() {
            return Err(format!("File too small to be a valid ELF64 binary: {}", filename).into());
        }

        let mut file_data = vec![0; file_size];
        file.read_exact(&mut file_data)?;

        // Parse and validate ELF header
        let ehdr: Elf64Ehdr = unsafe { std::ptr::read(file_data.as_ptr() as *const _) };

        // Magic number
        if ehdr.e_ident[0..4] != [0x7f, b'E', b'L', b'F'] {
            return Err("Not an ELF file (bad magic number)".into());
        }

        // 64-bit class
        if ehdr.e_ident[4] != 2 {
            return Err(format!("ELF is {}; recompile with riscv64-unknown-elf-gcc (EI_CLASS={})", if ehdr.e_ident[4] == 1 { "32-bit" } else { "unknown class" }, ehdr.e_ident[4]).into());
        }

        // Little-endian
        if ehdr.e_ident[5] != 1 {
            return Err(format!("ELF is big-endian; ensure the target triple is riscv64 (EI_DATA={})", ehdr.e_ident[5]).into());
        }

        // ELF version
        if ehdr.e_ident[6] != 1 {
            return Err(format!("Unknown ELF version (EI_VERSION={})", ehdr.e_ident[6]).into());
        }

        // RISC-V architecture
        if ehdr.e_machine != 0xF3 {
            return Err(format!("Not a RISC-V ELF (e_machine=0x{:x}); recompile targeting riscv64", ehdr.e_machine).into());
        }

        // Executable type
        if ehdr.e_type == 3 {
            return Err("ELF is a shared object / position-independent executable; relink as a static executable with -static -no-pie".into());
        }
        if ehdr.e_type != 2 {
            return Err(format!("ELF is not an executable (e_type={}); expected ET_EXEC (2)", ehdr.e_type).into());
        }

        // RISC-V ISA / ABI flags
        const EF_RISCV_RVC: u32 = 0x0001;
        const EF_RISCV_FLOAT_ABI_MASK: u32 = 0x0006;
        const EF_RISCV_RVE: u32 = 0x0008;

        if ehdr.e_flags & EF_RISCV_FLOAT_ABI_MASK != 0 {
            return Err(format!("ELF uses a hardware floating-point ABI (e_flags=0x{:x}); this VM implements RV64IM (integer only). Recompile with -march=rv64im -mabi=lp64", ehdr.e_flags).into());
        }

        if ehdr.e_flags & EF_RISCV_RVC != 0 {
            return Err(format!("ELF contains RISC-V Compressed (C) extension instructions (EF_RISCV_RVC set in e_flags=0x{:x}); this VM only handles 32-bit instructions. Recompile with -march=rv64im (omit 'c' from the march string) or add -mno-rvc", ehdr.e_flags).into());
        }

        if ehdr.e_flags & EF_RISCV_RVE != 0 {
            return Err(format!("ELF uses the RV32E reduced (16-register) integer ABI (EF_RISCV_RVE in e_flags=0x{:x}); recompile targeting riscv64", ehdr.e_flags).into());
        }

        // Program header table
        if ehdr.e_phoff == 0 || ehdr.e_phnum == 0 {
            return Err("ELF has no program headers; link as a static executable, not a relocatable object (.o)".into());
        }

        if usize::from(ehdr.e_phentsize) < std::mem::size_of::<Elf64Phdr>() {
            return Err(format!("ELF program header entry size too small (e_phentsize={}; expected >= {})", ehdr.e_phentsize, std::mem::size_of::<Elf64Phdr>()).into());
        }

        let phtab_end = ehdr.e_phoff + ehdr.e_phnum as u64 * ehdr.e_phentsize as u64;
        if phtab_end > file_size as u64 {
            return Err("ELF program header table extends beyond end of file".into());
        }

        // First pass: validate segments and compute address span
        let mut has_interp = false;
        let mut has_dynamic = false;
        let mut vaddr_min = !0u64;
        let mut vaddr_max = 0;
        let mut tls_tp = 0;

        for i in 0..ehdr.e_phnum {
            let phdr: Elf64Phdr = unsafe {
                std::ptr::read(
                    file_data
                        .as_ptr()
                        .add(ehdr.e_phoff as usize + i as usize * ehdr.e_phentsize as usize)
                        as *const _,
                )
            };

            match phdr.p_type {
                3 => has_interp = true,
                2 => has_dynamic = true,
                7 => tls_tp = phdr.p_vaddr,
                1 => {
                    if phdr.p_filesz > phdr.p_memsz {
                        return Err(format!(
                            "ELF PT_LOAD segment[{}]: p_filesz ({}) > p_memsz ({}) — malformed ELF",
                            i, phdr.p_filesz, phdr.p_memsz
                        )
                        .into());
                    }

                    if phdr.p_offset + phdr.p_filesz > file_size as u64 {
                        return Err(format!(
                            "ELF PT_LOAD segment[{}] file data extends beyond end of file",
                            i
                        )
                        .into());
                    }

                    vaddr_min = min(vaddr_min, phdr.p_vaddr);
                    vaddr_max = max(vaddr_max, phdr.p_vaddr + phdr.p_memsz);
                }
                _ => {}
            }
        }

        if has_interp {
            return Err("ELF requires a dynamic linker (PT_INTERP segment present); recompile and link with -static".into());
        }

        if has_dynamic {
            return Err("ELF contains dynamic linking information (PT_DYNAMIC segment present); recompile and link with -static".into());
        }

        if vaddr_min == !0 {
            return Err("ELF has no loadable (PT_LOAD) segments — nothing to execute".into());
        }

        if vaddr_max > max_size as u64 {
            return Err(format!("ELF virtual address span [0x{:x}, 0x{:x}) requires {} bytes which exceeds max_program_size={}; construct the VM with a larger max_program_size", vaddr_min, vaddr_max, vaddr_max, max_size).into());
        }

        if ehdr.e_entry < vaddr_min || ehdr.e_entry >= vaddr_max {
            return Err(format!("ELF entry point 0x{:x} lies outside the loaded virtual address range [0x{:x}, 0x{:x}); the binary may not have been linked correctly", ehdr.e_entry, vaddr_min, vaddr_max).into());
        }

        // Second pass: populate program image
        let mut prog = vec![0; vaddr_max as usize];

        for i in 0..ehdr.e_phnum {
            let phdr: Elf64Phdr = unsafe {
                std::ptr::read(
                    file_data
                        .as_ptr()
                        .add(ehdr.e_phoff as usize + i as usize * ehdr.e_phentsize as usize)
                        as *const _,
                )
            };

            if phdr.p_type != 1 || phdr.p_filesz == 0 {
                continue;
            }

            prog[phdr.p_vaddr as usize..(phdr.p_vaddr + phdr.p_filesz) as usize]
                .copy_from_slice(&file_data[phdr.p_offset as usize..(phdr.p_offset + phdr.p_filesz) as usize]);
        }

        Ok((prog, ehdr.e_entry, tls_tp))
    }
}

#[repr(C)]
struct Elf64Ehdr {
    e_ident: [U8; 16],
    e_type: u16,
    e_machine: u16,
    e_version: u32,
    e_entry: U64,
    e_phoff: U64,
    e_shoff: U64,
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
    p_offset: U64,
    p_vaddr: U64,
    p_paddr: U64,
    p_filesz: U64,
    p_memsz: U64,
    p_align: U64,
}

fn main() {
    // Code entry point; include example usage or testing here
}