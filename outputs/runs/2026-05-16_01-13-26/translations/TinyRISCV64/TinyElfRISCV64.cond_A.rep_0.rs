mod tinyriscv64 {
    use std::collections::HashMap;
    use std::fs::File;
    use std::io::{Read, Write};
    use std::time::{SystemTime, UNIX_EPOCH};

    type U64 = u64;
    type I64 = i64;
    type U8 = u8;
    
    #[repr(C)]
    struct VM {
        stack_size: usize,
        max_prog_size: usize,
        halted: bool,
        pc: U64,
        x: [U64; 32],
        program: Vec<U8>,
    }

    impl VM {
        fn new(stack_size: usize, max_program_size: usize) -> Self {
            Self {
                stack_size,
                max_prog_size: max_program_size,
                halted: false,
                pc: 0,
                x: [0; 32],
                program: Vec::new(),
            }
        }

        fn reset(&mut self) {
            self.halted = false;
            self.pc = 0;
            self.x = [0; 32];
        }

        fn mem_load<T>(&self, _addr: U64) -> T where T: Default + Copy {
            T::default()
        }

        fn mem_store<T>(&mut self, _addr: U64, _value: T) where T: Copy {}
    }

    pub struct ElfVM {
        vm: VM,
        fd_streams: HashMap<U64, Box<dyn std::io::Write + Send>>,
        tls_tp: U64,
    }

    impl ElfVM {
        pub fn new(stack_size: usize, max_program_size: usize) -> Self {
            Self {
                vm: VM::new(stack_size, max_program_size),
                fd_streams: HashMap::new(),
                tls_tp: 0,
            }
        }

        pub fn program_load(&mut self, prog_filename: &str) -> U64 {
            let (prog, entry, tp) = Self::load_elf(prog_filename, self.vm.max_prog_size).expect("Failed to load ELF");
            self.tls_tp = tp;
            self.vm.program = prog;
            self.reset();
            entry
        }

        fn reset(&mut self) {
            self.vm.reset();
            if self.tls_tp != 0 {
                self.vm.x[4] = self.tls_tp;
            }
        }

        pub fn map_fd(&mut self, fd: U64, stream: Box<dyn std::io::Write + Send>) {
            self.fd_streams.insert(fd, stream);
        }

        fn mem_read_str(&mut self, addr: U64) -> String {
            let mut s = String::new();
            loop {
                let c = self.vm.mem_load::<U8>(addr);
                if c == 0 {
                    break;
                }
                s.push(c as char);
            }
            s
        }

        fn handle_semihost(&mut self) {
            let op = self.vm.x[10];
            let arg = self.vm.x[11];

            let argv_fetched = {
                let argv = |n: usize| -> U64 {
                    self.vm.mem_load(arg + n as U64 * 8)
                };
                (argv(0), argv(1), argv(2))
            };

            match op {
                0x01 => {
                    let path = self.mem_read_str(argv_fetched.0);
                    let mode = argv_fetched.1;

                    if path == ":tt" {
                        if mode < 4 {
                            self.vm.x[10] = 0;
                        } else if mode < 8 {
                            self.vm.x[10] = 1;
                        } else if mode < 12 {
                            self.vm.x[10] = 2;
                        } else {
                            self.vm.x[10] = u64::MAX;
                        }
                        return;
                    }

                    self.vm.x[10] = u64::MAX;
                },
                0x02 => {
                    let fd = argv_fetched.0;
                    self.fd_streams.remove(&fd);
                    self.vm.x[10] = 0;
                },
                0x03 => {
                    let c = self.vm.mem_load::<U8>(arg) as char;
                    if let Some(stream) = self.fd_streams.get_mut(&1) {
                        let _ = stream.write(&[c as u8]);
                    }
                    self.vm.x[10] = 0;
                },
                0x04 => {
                    let s = self.mem_read_str(arg);
                    if let Some(stream) = self.fd_streams.get_mut(&1) {
                        let _ = stream.write(s.as_bytes());
                    }
                    self.vm.x[10] = 0;
                },
                0x05 => {
                    let fd = argv_fetched.0;
                    let buf = argv_fetched.1;
                    let mut len = argv_fetched.2;

                    if let Some(stream) = self.fd_streams.get_mut(&fd) {
                        let mut i = 0;
                        while len > 0 && stream.write(&[self.vm.mem_load::<U8>(buf + i)]).is_ok() {
                            i += 1;
                            len -= 1;
                        }
                    }

                    self.vm.x[10] = len;
                },
                0x06 => {
                    let fd = argv_fetched.0;
                    let buf = argv_fetched.1;
                    let len = argv_fetched.2;

                    if let Some(stream) = self.fd_streams.get_mut(&fd) {
                        let mut tmp = vec![0u8; len as usize];
                        match stream.write(&mut tmp) {
                            Ok(n) => {
                                for i in 0..n {
                                    self.vm.mem_store(buf + i as u64, tmp[i]);
                                }
                                self.vm.x[10] = len - n as u64;
                            }
                            Err(_) => self.vm.x[10] = len,
                        }
                    } else {
                        self.vm.x[10] = len;
                    }
                },
                0x07 => {},
                0x09 => {
                    let fd = argv_fetched.0;
                    self.vm.x[10] = if fd <= 2 { 1 } else { 0 };
                },
                0x0a => {},
                0x0c => {},
                0x11 => {
                    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                    self.vm.x[10] = now.as_secs() as u64;
                },
                0x18 => {
                    self.vm.halted = true;
                    self.vm.x[10] = 0;
                },
                0x30 => {
                    self.vm.halted = true;
                    self.vm.x[10] = argv_fetched.1;
                }
                _ => {
                    let message = format!(
                        "Unsupported semihosting operation 0x{:x} at pc=0x{:x}",
                        op, self.vm.pc - 4
                    );
                    eprintln!("{}", message);
                    panic!("{}", message);
                }
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
                },
                9 | 215 | 222 | 214 | 226 | 233 | 216 | 219 | 228 => {
                    self.vm.x[10] = 38;
                },
                57 => {
                    self.fd_streams.remove(&a0);
                    self.vm.x[10] = 0;
                },
                62 => {},
                63 => {},
                64 => {
                    if let Some(stream) = self.fd_streams.get_mut(&a0) {
                        let mut buffer = vec![0; a2 as usize];
                        for i in 0..a2 {
                            buffer[i as usize] = self.vm.mem_load::<U8>(a1 + i as u64);
                        }
                        let res = stream.write(&buffer).is_ok();
                        self.vm.x[10] = if res { a2 } else { 5 };
                    } else {
                        self.vm.x[10] = 9;
                    }
                },
                160 | 278 => {
                    self.vm.x[10] = 38;
                },
                96 => {
                    self.vm.x[10] = 1;
                },
                220 | 221 => {
                    self.vm.x[10] = 38;
                },
                _ => {
                    let message = format!(
                        "ecall: unsupported syscall number {} (a0={}, a1=0x{:x}, a2={}) at pc=0x{:x}",
                        num, a0, a1, a2, self.vm.pc - 4
                    );
                    eprintln!("{}", message);
                    panic!("{}", message);
                },
            }
        }

        fn load_elf(filename: &str, max_size: usize) -> Result<(Vec<u8>, u64, u64), String> {
            let mut fin = File::open(filename).map_err(|_| format!("Failed to open ELF file: {}", filename))?;
            let file_size = fin.metadata().map_err(|_| "Failed to get metadata")?.len() as usize;

            if file_size < std::mem::size_of::<Elf64Ehdr>() {
                return Err(format!("File too small to be a valid ELF64 binary: {}", filename));
            }

            let mut file_data = vec![0u8; file_size];
            fin.read(&mut file_data).map_err(|_| "Failed to read ELF file")?;

            let ehdr: Elf64Ehdr = unsafe { std::ptr::read(file_data.as_ptr() as *const Elf64Ehdr) };
            
            if ehdr.e_ident[0] != 0x7f || ehdr.e_ident[1] != b'E' ||
               ehdr.e_ident[2] != b'L' || ehdr.e_ident[3] != b'F' {
                return Err("Not an ELF file (bad magic number)".to_string());
            }

            if ehdr.e_ident[4] != 2 {
                return Err(format!(
                    "ELF is {}; recompile with riscv64-unknown-elf-gcc (EI_CLASS={})",
                    if ehdr.e_ident[4] == 1 { "32-bit" } else { "unknown class" }, ehdr.e_ident[4]
                ));
            }

            if ehdr.e_ident[5] != 1 {
                return Err(format!(
                    "ELF is big-endian; ensure the target triple is riscv64 (EI_DATA={})",
                    ehdr.e_ident[5]
                ));
            }

            if ehdr.e_ident[6] != 1 {
                return Err(format!("Unknown ELF version (EI_VERSION={})", ehdr.e_ident[6]));
            }

            if ehdr.e_machine != 0xF3 {
                return Err(format!(
                    "Not a RISC-V ELF (e_machine=0x{:x}); recompile targeting riscv64 (e.g. riscv64-unknown-elf-gcc)",
                    ehdr.e_machine
                ));
            }

            match ehdr.e_type {
                2 => {},
                3 => {
                    return Err("ELF is a shared object / position-independent executable; relink as a static executable with -static -no-pie".to_string());
                },
                _ => {
                    return Err(format!(
                        "ELF is not an executable (e_type={}); expected ET_EXEC (2)",
                        ehdr.e_type
                    ));
                },
            }

            let mut prog = vec![0u8; max_size];
            prog.splice(..file_data.len(), file_data.iter().cloned());

            Ok((prog, ehdr.e_entry, 0))
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
}

fn main() {
    // This function is intentionally left empty.
}