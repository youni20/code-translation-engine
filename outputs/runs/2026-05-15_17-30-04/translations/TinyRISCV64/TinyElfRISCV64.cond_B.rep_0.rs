mod tinyriscv64 {
    use std::collections::HashMap;
    use std::fs::File;
    use std::io::{Read, Write, Seek, SeekFrom};
    use std::time::SystemTime;
    use std::rc::Rc;
    use std::cell::RefCell;

    type U64 = u64;
    type I64 = i64;
    type U32 = u32;
    type U16 = u16;
    type U8 = u8;

    pub struct VM {
        stack_size: usize,
        max_program_size: usize,
        x: [U64; 32],
        halted: bool,
        pc: U64,
    }

    impl VM {
        pub fn new(stack_size: usize, max_program_size: usize) -> Self {
            VM {
                stack_size,
                max_program_size,
                x: [0; 32],
                halted: false,
                pc: 0,
            }
        }

        pub fn reset(&mut self) {
            self.x = [0; 32];
            self.halted = false;
        }
    }
    
    pub struct ElfVM {
        vm: VM,
        fd_streams: HashMap<U64, Rc<RefCell<dyn Stream>>>,
        tls_tp: U64,
    }

    trait Stream: Read + Write + Seek {}

    impl<T: Read + Write + Seek> Stream for T {}

    impl ElfVM {
        pub fn new(stack_size: usize, max_program_size: usize) -> Self {
            ElfVM {
                vm: VM::new(stack_size, max_program_size),
                fd_streams: HashMap::new(),
                tls_tp: 0,
            }
        }

        pub fn program_load(&mut self, prog_filename: &str) -> Result<U64, String> {
            let (_prog, entry, tp) = Self::load_elf(prog_filename, self.vm.max_program_size)?;
            self.tls_tp = tp;
            self.vm.reset();
            Ok(entry)
        }

        pub fn reset(&mut self) {
            self.vm.reset();
            if self.tls_tp != 0 {
                self.vm.x[4] = self.tls_tp;
            }
        }

        pub fn map_fd(&mut self, fd: U64, stream: Rc<RefCell<dyn Stream>>) {
            self.fd_streams.insert(fd, stream);
        }

        fn mem_read_str(&self, _addr: U64) -> String {
            String::new()
        }

        fn mem_load<T: Default>(&self, _addr: U64) -> T {
            T::default()
        }

        fn mem_store<T>(&self, _addr: U64, _value: T) {}

        fn handle_semihost(&mut self) {
            let op = self.vm.x[10];
            let arg = self.vm.x[11];

            let argv = |n: usize| -> U64 {
                self.mem_load(arg + n as U64 * 8)
            };
            
            match op {
                0x01 => {
                    let path = self.mem_read_str(argv(0));
                    let mode = argv(1);

                    if path == ":tt" {
                        self.vm.x[10] = match mode {
                            0..=3 => 0,
                            4..=7 => 1,
                            8..=11 => 2,
                            _ => u64::MAX,
                        };
                        return;
                    }

                    self.vm.x[10] = u64::MAX;
                }
                0x02 => {
                    let fd = argv(0);
                    self.fd_streams.remove(&fd);
                    self.vm.x[10] = 0;
                }
                0x03 => {
                    let c = self.mem_load::<u8>(arg) as char;
                    if let Some(stream) = self.fd_streams.get(&1) {
                        let _ = stream.borrow_mut().write_all(&[c as u8]);
                    }
                    self.vm.x[10] = 0;
                }
                0x04 => {
                    let s = self.mem_read_str(arg);
                    if let Some(stream) = self.fd_streams.get(&1) {
                        let _ = stream.borrow_mut().write_all(s.as_bytes());
                    }
                    self.vm.x[10] = 0;
                }
                0x05 => {
                    let fd = argv(0);
                    let buf = argv(1);
                    let mut len = argv(2);

                    if let Some(stream) = self.fd_streams.get(&fd) {
                        let mut i = 0;
                        while len > 0 {
                            let c = self.mem_load::<u8>(buf + i);
                            let _ = stream.borrow_mut().write_all(&[c]);
                            i += 1;
                            len -= 1;
                        }
                    }
                    self.vm.x[10] = len;
                }
                0x06 => {
                    let fd = argv(0);
                    let buf = argv(1);
                    let len = argv(2);

                    if let Some(stream) = self.fd_streams.get(&fd) {
                        let mut tmp = vec![0u8; len as usize];
                        let n = match stream.borrow_mut().read(&mut tmp) {
                            Ok(n) => n as U64,
                            Err(_) => 0 as U64,
                        };
                        for i in 0..n {
                            self.mem_store(buf + i, tmp[i as usize]);
                        }
                        self.vm.x[10] = len - n;
                    } else {
                        self.vm.x[10] = len;
                    }
                }
                0x07 => {
                    if let Some(stream) = self.fd_streams.get(&0) {
                        let mut buffer = [0; 1];
                        match stream.borrow_mut().read(&mut buffer) {
                            Ok(1) => self.vm.x[10] = buffer[0] as U64,
                            _ => self.vm.x[10] = 0xFFFFFFFFFFFFFFFF,
                        }
                    } else {
                        self.vm.x[10] = 0xFFFFFFFFFFFFFF04;
                    }
                }
                0x09 => {
                    let fd = argv(0);
                    self.vm.x[10] = if fd <= 2 { 1 } else { 0 };
                }
                0x0a => {
                    let fd = argv(0);
                    let off = argv(1) as i64;
                    if let Some(stream) = self.fd_streams.get(&fd) {
                        let _ = stream.borrow_mut().seek(SeekFrom::Start(off as u64));
                        self.vm.x[10] = if stream.borrow_mut().seek(SeekFrom::Current(0)).is_ok() { 0 } else { u64::MAX };
                    } else {
                        self.vm.x[10] = u64::MAX;
                    }
                }
                0x0c => {
                    let fd = argv(0);
                    if let Some(stream) = self.fd_streams.get(&fd) {
                        let mut s = stream.borrow_mut();
                        let current = s.seek(SeekFrom::Current(0)).unwrap_or(0);
                        let len = s.seek(SeekFrom::End(0)).unwrap_or(0);
                        let _ = s.seek(SeekFrom::Start(current));
                        self.vm.x[10] = len as U64;
                    } else {
                        self.vm.x[10] = u64::MAX;
                    }
                }
                0x11 => {
                    self.vm.x[10] = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
                }
                0x18 => {
                    self.vm.halted = true;
                    self.vm.x[10] = 0;
                }
                0x30 => {
                    let arg1 = argv(1);
                    self.vm.halted = true;
                    self.vm.x[10] = arg1;
                }
                _ => panic!("Unsupported semihosting operation 0x{:x} at pc=0x{:x}", op, self.vm.pc - 4),
            }
        }

        fn handle_ecall(&mut self) {
            let num = self.vm.x[17];
            let a0 = self.vm.x[10];
            let a1 = self.vm.x[11];
            let a2 = self.vm.x[12];

            match num {
                93 | 94 => {
                    self.vm.halted = true;
                }
                9 | 215 | 222 | 214 | 226 | 233 | 216 | 219 | 228 => {
                    self.vm.x[10] = u64::MAX - 38; 
                }
                57 => {
                    self.fd_streams.remove(&a0);
                    self.vm.x[10] = 0;
                }
                62 => {
                    if let Some(stream) = self.fd_streams.get(&a0) {
                        let dirs = [SeekFrom::Start(0), SeekFrom::Current(0), SeekFrom::End(0)];
                        if a2 > 2 {
                            self.vm.x[10] = u64::MAX - 22;
                        } else {
                            let _ = stream.borrow_mut().seek(dirs[a2 as usize]);
                            self.vm.x[10] = if stream.borrow_mut().seek(dirs[a2 as usize]).is_ok() {
                                stream.borrow_mut().seek(SeekFrom::Current(0)).unwrap_or(0) as U64
                            } else {
                                u64::MAX - 29
                            };
                        }
                    } else {
                        self.vm.x[10] = u64::MAX - 9;
                    }
                }
                63 => {
                    if let Some(stream) = self.fd_streams.get(&a0) {
                        let mut buf = vec![0u8; a2 as usize];
                        let _ = stream.borrow_mut().read(&mut buf);
                        let n = buf.len() as U64;
                        for i in 0..n {
                            self.mem_store(a1 + i, buf[i as usize]);
                        }
                        self.vm.x[10] = n;
                    } else {
                        self.vm.x[10] = u64::MAX - 9;
                    }
                }
                64 => {
                    if let Some(stream) = self.fd_streams.get(&a0) {
                        let mut buf = vec![0u8; a2 as usize];
                        for i in 0..a2 {
                            buf[i as usize] = self.mem_load::<u8>(a1 + i);
                        }
                        self.vm.x[10] = if stream.borrow_mut().write(&buf).is_ok() {
                            a2
                        } else {
                            u64::MAX - 5
                        };
                    } else {
                        self.vm.x[10] = u64::MAX - 9;
                    }
                }
                56 | 65 | 66 | 79 | 80 => {
                    self.vm.x[10] = u64::MAX - 38;
                }
                160 => {
                    self.vm.x[10] = u64::MAX - 38;
                }
                278 => {
                    self.vm.x[10] = u64::MAX - 38;
                }
                113 | 169 => {
                    self.vm.x[10] = u64::MAX - 22;
                }
                174 | 175 | 176 | 177 => {
                    self.vm.x[10] = 0;
                }
                96 => {
                    self.vm.x[10] = 1;
                }
                99 | 100 | 261 | 132 | 134 | 135 => {
                    self.vm.x[10] = 0;
                }
                220 | 221 => {
                    self.vm.x[10] = u64::MAX - 38;
                }
                _ => panic!("ecall: unsupported syscall number {} (a0={}, a1=0x{:x}, a2={}) at pc=0x{:x}",
                    num, a0, a1, a2, self.vm.pc - 4),
            }
        }

        fn load_elf(filename: &str, max_size: usize) -> Result<(Vec<U8>, U64, U64), String> {
            let mut fin = File::open(filename).or_else(|_| Err(format!("Failed to open ELF file: {}", filename)))?;
            let file_size = fin.metadata().map(|m| m.len() as usize).unwrap_or(0);

            if file_size < std::mem::size_of::<Elf64Ehdr>() {
                return Err(format!("File too small to be a valid ELF64 binary: {}", filename));
            }

            let mut file_data = vec![0u8; file_size];
            fin.read_exact(&mut file_data).or_else(|_| Err("Error reading ELF file".to_string()))?;

            let ehdr: Elf64Ehdr = unsafe { std::ptr::read(file_data.as_ptr() as *const _) };

            if &ehdr.e_ident[0..4] != b"\x7FELF" {
                return Err("Not an ELF file (bad magic number)".to_string());
            }

            if ehdr.e_ident[4] != 2 {
                return Err(format!(
                    "ELF is {}; recompile with riscv64-unknown-elf-gcc (EI_CLASS={})",
                    if ehdr.e_ident[4] == 1 { "32-bit" } else { "unknown class" },
                    ehdr.e_ident[4]
                ));
            }

            if ehdr.e_ident[5] != 1 {
                return Err(format!(
                    "ELF is big-endian; ensure the target triple is riscv64 (EI_DATA={})",
                    ehdr.e_ident[5]
                ));
            }

            if ehdr.e_ident[6] != 1 {
                return Err(format!(
                    "Unknown ELF version (EI_VERSION={})",
                    ehdr.e_ident[6]
                ));
            }

            if ehdr.e_machine != 0xF3 {
                return Err(format!(
                    "Not a RISC-V ELF (e_machine=0x{:x}); recompile targeting riscv64 (e.g. riscv64-unknown-elf-gcc)",
                    ehdr.e_machine
                ));
            }

            if ehdr.e_type == 3 {
                return Err(
                    "ELF is a shared object / position-independent executable; relink as a static executable with -static -no-pie"
                        .to_string(),
                );
            }

            if ehdr.e_type != 2 {
                return Err(format!(
                    "ELF is not an executable (e_type={}); expected ET_EXEC (2)",
                    ehdr.e_type
                ));
            }

            const EF_RISCV_RVC: U32 = 0x0001;
            const EF_RISCV_FLOAT_ABI_MASK: U32 = 0x0006;
            const EF_RISCV_RVE: U32 = 0x0008;

            if ehdr.e_flags & EF_RISCV_FLOAT_ABI_MASK != 0 {
                return Err(format!(
                    "ELF uses a hardware floating-point ABI (e_flags=0x{:x}); this VM implements RV64IM (integer only). Recompile with -march=rv64im -mabi=lp64",
                    ehdr.e_flags
                ));
            }

            if ehdr.e_flags & EF_RISCV_RVC != 0 {
                return Err(format!(
                    "ELF contains RISC-V Compressed (C) extension instructions (EF_RISCV_RVC set in e_flags=0x{:x}); this VM only handles 32-bit instructions. Recompile with -march=rv64im (omit 'c' from the march string) or add -mno-rvc",
                    ehdr.e_flags
                ));
            }

            if ehdr.e_flags & EF_RISCV_RVE != 0 {
                return Err(format!(
                    "ELF uses the RV32E reduced (16-register) integer ABI (EF_RISCV_RVE in e_flags=0x{:x}); recompile targeting riscv64",
                    ehdr.e_flags
                ));
            }

            if ehdr.e_phoff == 0 || ehdr.e_phnum == 0 {
                return Err(
                    "ELF has no program headers; link as a static executable, not a relocatable object (.o)"
                        .to_string(),
                );
            }

            if usize::from(ehdr.e_phentsize) < std::mem::size_of::<Elf64Phdr>() {
                return Err(format!(
                    "ELF program header entry size too small (e_phentsize={}; expected >= {})",
                    ehdr.e_phentsize,
                    std::mem::size_of::<Elf64Phdr>()
                ));
            }

            let phtab_end = ehdr.e_phoff + (ehdr.e_phnum as U64) * (ehdr.e_phentsize as U64);
            if phtab_end > file_data.len() as U64 {
                return Err("ELF program header table extends beyond end of file".to_string());
            }

            let mut has_interp = false;
            let mut has_dynamic = false;
            let mut vaddr_min = U64::MAX;
            let mut vaddr_max = 0;
            let mut tls_tp = 0; 

            for i in 0..ehdr.e_phnum {
                let phdr_offset = ehdr.e_phoff + (i as U64) * (ehdr.e_phentsize as U64);
                let phdr: Elf64Phdr = unsafe { std::ptr::read(file_data[phdr_offset as usize..].as_ptr() as *const _) };

                match phdr.p_type {
                    3 => has_interp = true,
                    2 => has_dynamic = true,
                    7 => tls_tp = phdr.p_vaddr,
                    1 => {
                        if phdr.p_filesz > phdr.p_memsz {
                            return Err(format!(
                                "ELF PT_LOAD segment[{}]: p_filesz ({}) > p_memsz ({}) — malformed ELF",
                                i, phdr.p_filesz, phdr.p_memsz
                            ));
                        }

                        if phdr.p_offset + phdr.p_filesz > file_data.len() as U64 {
                            return Err(format!(
                                "ELF PT_LOAD segment[{}] file data extends beyond end of file",
                                i
                            ));
                        }

                        vaddr_min = std::cmp::min(vaddr_min, phdr.p_vaddr);
                        vaddr_max = std::cmp::max(vaddr_max, phdr.p_vaddr + phdr.p_memsz);
                    }
                    _ => {}
                };
            }

            if has_interp {
                return Err(
                    "ELF requires a dynamic linker (PT_INTERP segment present); recompile and link with -static"
                        .to_string(),
                );
            }

            if has_dynamic {
                return Err(
                    "ELF contains dynamic linking information (PT_DYNAMIC segment present); recompile and link with -static"
                        .to_string(),
                );
            }

            if vaddr_min == U64::MAX {
                return Err(
                    "ELF has no loadable (PT_LOAD) segments — nothing to execute".to_string(),
                );
            }

            if vaddr_max > max_size as U64 {
                return Err(format!(
                    "ELF virtual address span [0x{:x}, 0x{:x}) requires {} bytes which exceeds max_program_size={}; construct the VM with a larger max_program_size",
                    vaddr_min, vaddr_max, vaddr_max, max_size
                ));
            }

            if ehdr.e_entry < vaddr_min || ehdr.e_entry >= vaddr_max {
                return Err(format!(
                    "ELF entry point 0x{:x} lies outside the loaded virtual address range [0x{:x}, 0x{:x}); the binary may not have been linked correctly",
                    ehdr.e_entry, vaddr_min, vaddr_max
                ));
            }

            let mut prog = vec![0u8; vaddr_max as usize];

            for i in 0..ehdr.e_phnum {
                let phdr_offset = ehdr.e_phoff + (i as U64) * (ehdr.e_phentsize as U64);
                let phdr: Elf64Phdr = unsafe { std::ptr::read(file_data[phdr_offset as usize..].as_ptr() as *const _) };

                if phdr.p_type != 1 || phdr.p_filesz == 0 {
                    continue;
                }

                let dest = &mut prog[phdr.p_vaddr as usize..];
                let src = &file_data[phdr.p_offset as usize..];
                dest[..phdr.p_filesz as usize].copy_from_slice(&src[..phdr.p_filesz as usize]);
            }

            Ok((prog, ehdr.e_entry, tls_tp))
        }
    }

    #[repr(C)]
    struct Elf64Ehdr {
        e_ident: [U8; 16],
        e_type: U16,
        e_machine: U16,
        e_version: U32,
        e_entry: U64,
        e_phoff: U64,
        e_shoff: U64,
        e_flags: U32,
        e_ehsize: U16,
        e_phentsize: U16,
        e_phnum: U16,
        e_shentsize: U16,
        e_shnum: U16,
        e_shstrndx: U16,
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
}

fn main() {
    // Entry point added to compile the module
}