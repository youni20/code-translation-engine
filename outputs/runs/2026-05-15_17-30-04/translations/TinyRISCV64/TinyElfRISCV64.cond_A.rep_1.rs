/*
 * TinyRISCV64 extension to add elf loading and syscall ABIs
 *
 * https://github.com/neilstephens/TinyRISCV64
 *
 * MIT License
 *
 * Copyright (c) 2025 Neil Stephens
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

mod tiny_riscv64 {
    use std::collections::HashMap;
    use std::fs::File;
    use std::io::{Read, Write};
    use std::sync::{Arc, Mutex};
    use std::time::SystemTime;
    use std::mem;
    use std::path::Path;

    type U64 = u64;
    type U8 = u8;
    type U32 = u32;
    type U16 = u16;
    type I64 = i64;

    pub trait VM {
        fn reset(&mut self);

        fn program_load(&mut self, prog_filename: &str) -> U64;
    }

    pub struct ElfVM {
        fd_streams: HashMap<U64, Arc<Mutex<dyn Write + Send>>>,
        tls_tp: U64,
        x: [U64; 32],
        pc: U64,
        halted: bool,
        program: Vec<u8>,
        max_prog_size: usize,
    }

    impl ElfVM {
        pub fn new(_stack_size: usize, max_program_size: usize) -> ElfVM {
            ElfVM {
                fd_streams: HashMap::new(),
                tls_tp: 0,
                x: [0; 32],
                pc: 0,
                halted: false,
                program: Vec::new(),
                max_prog_size: max_program_size,
            }
        }

        fn map_fd(&mut self, fd: U64, stream: Arc<Mutex<dyn Write + Send>>) {
            self.fd_streams.insert(fd, stream);
        }

        fn mem_read_str(&self, addr: U64) -> String {
            let mut addr = addr as usize;
            let mut s = String::new();
            loop {
                let c = self.mem_load::<U8>(addr);
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

            let argv = |n| self.mem_load::<U64>((arg + n * 8u64) as usize);

            match op {
                0x01 => {
                    let path = self.mem_read_str(argv(0));
                    let mode = argv(1);

                    if path == ":tt" {
                        self.x[10] = match mode {
                            m if m < 4 => 0,
                            m if m < 8 => 1,
                            m if m < 12 => 2,
                            _ => u64::MAX,
                        };
                        return;
                    }

                    self.x[10] = u64::MAX;
                }
                0x02 => {
                    let fd = argv(0);
                    self.fd_streams.remove(&fd);
                    self.x[10] = 0;
                }
                0x03 => {
                    let c = self.mem_load::<U8>(arg.try_into().unwrap()) as char;
                    if let Some(stream) = self.fd_streams.get_mut(&1) {
                        let mut locked_stream = stream.lock().unwrap();
                        write!(locked_stream, "{}", c).unwrap();
                    }
                    self.x[10] = 0;
                }
                0x04 => {
                    let s = self.mem_read_str(arg);
                    if let Some(stream) = self.fd_streams.get_mut(&1) {
                        let mut locked_stream = stream.lock().unwrap();
                        write!(locked_stream, "{}", s).unwrap();
                    }
                    self.x[10] = 0;
                }
                0x05 => {
                    let fd = argv(0);
                    let buf = argv(1);
                    let len = argv(2);

                    if let Some(stream) = self.fd_streams.get_mut(&fd) {
                        let mut locked_stream = stream.lock().unwrap();
                        let mut all_bytes_written = true;
                        for i in 0..len {
                            let b = self.mem_load::<U8>((buf + i).try_into().unwrap());
                            if locked_stream.write(&[b]).is_err() {
                                all_bytes_written = false;
                                break;
                            }
                        }
                        self.x[10] = if all_bytes_written { 0 } else { len };
                    } else {
                        self.x[10] = len;
                    }
                }
                0x06 => {
                    let fd = argv(0);
                    let buf = argv(1);
                    let len = argv(2);

                    if let Some(stream) = self.fd_streams.get_mut(&fd) {
                        let tmp = vec![0; len as usize];
                        for i in 0..len {
                            self.mem_store((buf + i).try_into().unwrap(), tmp[i as usize]);
                        }
                        self.x[10] = 0;
                    } else {
                        self.x[10] = len;
                    }
                }
                0x07 => {
                    if let Some(stream) = self.fd_streams.get_mut(&0) {
                        let mut locked_stream = stream.lock().unwrap();
                        let mut buf = [0; 1];
                        if locked_stream.write(&mut buf).is_ok() {
                            self.x[10] = buf[0] as U64;
                            return;
                        }
                    }
                    self.x[10] = 0xFFFFFFFFFFFFFF04u64;
                }
                0x09 => {
                    let fd = argv(0);
                    self.x[10] = if fd <= 2 { 1 } else { 0 };
                }
                0x0a => {
                    let _fd = argv(0);
                    let _off = argv(1) as I64;
                    self.x[10] = u64::MAX;
                }
                0x0c => {
                    let _fd = argv(0);
                    self.x[10] = u64::MAX;
                }
                0x11 => {
                    self.x[10] = SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                }
                0x18 => {
                    self.halted = true;
                    self.x[10] = 0;
                }
                0x30 => {
                    let param = argv(1);
                    self.halted = true;
                    self.x[10] = param;
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
                    self.x[10] = 0;
                }
                9 | 215 | 222 | 214 | 226 | 233 | 216 | 219 | 228 => {
                    self.x[10] = -38i64 as U64;
                }
                57 => {
                    self.fd_streams.remove(&a0);
                    self.x[10] = 0;
                }
                62 => {
                    self.x[10] = -9i64 as U64;
                }
                63 => {
                    self.x[10] = -9i64 as U64;
                }
                64 => {
                    let buf: Vec<U8> = {
                        let mut tmp_buf = Vec::with_capacity(a2 as usize);
                        for i in 0..a2 {
                            self.fd_streams.get(&a0); // Establish borrow scope
                            let b = self.mem_load((a1 + i) as usize);
                            tmp_buf.push(b);
                        }
                        tmp_buf
                    };

                    let fd_stream = self.fd_streams.get_mut(&a0);
                    
                    if let Some(stream) = fd_stream {
                        if stream.lock().unwrap().write(&buf).is_ok() {
                            self.x[10] = a2;
                        } else {
                            self.x[10] = -5i64 as U64;
                        }
                    } else {
                        self.x[10] = -9i64 as U64;
                    }
                }
                160 => {
                    self.x[10] = -38i64 as U64;
                }
                278 => {
                    self.x[10] = a1;
                }
                113 | 169 => {
                    self.x[10] = -22i64 as U64;
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
                    self.x[10] = -38i64 as U64;
                }
                _ => {
                    panic!(
                        "ecall: unsupported syscall number {} (a0={}, a1=0x{:x}, a2={}) at pc=0x{:x}",
                        num, a0, a1, a2, self.pc - 4
                    );
                }
            }
        }

        fn load_elf(filename: &str, max_size: usize) -> Result<(Vec<u8>, U64, U64), String> {
            let path = Path::new(filename);
            let mut file = File::open(path).map_err(|_| format!("Failed to open ELF file: {}", filename))?;
        
            let file_size = file.metadata().map(|m| m.len() as usize).unwrap_or(0);
            if file_size < mem::size_of::<Elf64Ehdr>() {
                return Err(format!("File too small to be a valid ELF64 binary: {}", filename));
            }
        
            let mut file_data = vec![0u8; file_size];
            file.read_exact(&mut file_data).map_err(|_| format!("Failed to read ELF file: {}", filename))?;
        
            let ehdr: Elf64Ehdr = unsafe { std::ptr::read(file_data.as_ptr() as *const Elf64Ehdr) };
        
            if ehdr.e_ident[0] != 0x7f
                || ehdr.e_ident[1] != b'E'
                || ehdr.e_ident[2] != b'L'
                || ehdr.e_ident[3] != b'F'
            {
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
                return Err("ELF is a shared object / position-independent executable; relink as a static executable with -static -no-pie".to_string());
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
                return Err("ELF has no program headers; link as a static executable, not a relocatable object (.o)".to_string());
            }
        
            if ehdr.e_phentsize < mem::size_of::<Elf64Phdr>() as u16 {
                return Err(format!(
                    "ELF program header entry size too small (e_phentsize={}; expected >= {})",
                    ehdr.e_phentsize,
                    mem::size_of::<Elf64Phdr>()
                ));
            }
        
            let phtab_end = ehdr.e_phoff + U64::from(ehdr.e_phnum) * U64::from(ehdr.e_phentsize);
            if phtab_end > file_size as u64 {
                return Err("ELF program header table extends beyond end of file".to_string());
            }
        
            let mut has_interp = false;
            let mut has_dynamic = false;
            let mut vaddr_min = u64::MAX;
            let mut vaddr_max = 0;
            let mut tls_tp = 0;
        
            for i in 0..ehdr.e_phnum {
                let phdr: Elf64Phdr = unsafe {
                    let ptr = file_data.as_ptr().add(ehdr.e_phoff as usize + i as usize * ehdr.e_phentsize as usize) as *const Elf64Phdr;
                    std::ptr::read(ptr)
                };
        
                match phdr.p_type {
                    3 => has_interp = true,   
                    2 => has_dynamic = true,  
                    7 => tls_tp = phdr.p_vaddr,
                    1 => {
                        if phdr.p_filesz > phdr.p_memsz {
                            return Err(format!(
                                "ELF PT_LOAD segment[{}]: p_filesz ({}) > p_memsz ({}) — malformed ELF",
                                i,
                                phdr.p_filesz,
                                phdr.p_memsz
                            ));
                        }
                        if phdr.p_offset + phdr.p_filesz > file_size as u64 {
                            return Err(format!(
                                "ELF PT_LOAD segment[{}] file data extends beyond end of file",
                                i
                            ));
                        }
                        vaddr_min = vaddr_min.min(phdr.p_vaddr);
                        vaddr_max = vaddr_max.max(phdr.p_vaddr + phdr.p_memsz);
                    }
                    _ => {}
                }
            }
        
            if has_interp {
                return Err("ELF requires a dynamic linker (PT_INTERP segment present); recompile and link with -static".to_string());
            }
        
            if has_dynamic {
                return Err("ELF contains dynamic linking information (PT_DYNAMIC segment present); recompile and link with -static".to_string());
            }
        
            if vaddr_min == u64::MAX {
                return Err("ELF has no loadable (PT_LOAD) segments — nothing to execute".to_string());
            }
        
            if vaddr_max > max_size as u64 {
                return Err(format!(
                    "ELF virtual address span [0x{:x}, 0x{:x}) requires {} bytes which exceeds max_program_size={}; construct the VM with a larger max_program_size",
                    vaddr_min,
                    vaddr_max,
                    vaddr_max,
                    max_size
                ));
            }
        
            if ehdr.e_entry < vaddr_min || ehdr.e_entry >= vaddr_max {
                return Err(format!(
                    "ELF entry point 0x{:x} lies outside the loaded virtual address range [0x{:x}, 0x{:x}); the binary may not have been linked correctly",
                    ehdr.e_entry,
                    vaddr_min,
                    vaddr_max
                ));
            }
        
            let mut prog = vec![0u8; vaddr_max as usize];
        
            for i in 0..ehdr.e_phnum {
                let phdr: Elf64Phdr = unsafe {
                    let ptr = file_data.as_ptr().add(ehdr.e_phoff as usize + i as usize * ehdr.e_phentsize as usize) as *const Elf64Phdr;
                    std::ptr::read(ptr)
                };
        
                if phdr.p_type == 1 && phdr.p_filesz > 0 {
                    let dest = unsafe { prog.as_mut_ptr().add(phdr.p_vaddr as usize) };
                    let src = unsafe { file_data.as_ptr().add(phdr.p_offset as usize) };
                    unsafe {
                        std::ptr::copy_nonoverlapping(src, dest, phdr.p_filesz as usize);
                    }
                }
            }
        
            Ok((prog, ehdr.e_entry, tls_tp))
        }


        fn mem_load<T: Copy>(&self, _addr: usize) -> T {
            unimplemented!("mem_load should be implemented based on VM specifics")
        }

        fn mem_store<T: Copy>(&self, _addr: usize, _val: T) {
            unimplemented!("mem_store should be implemented based on VM specifics")
        }
    }

    impl VM for ElfVM {
        fn reset(&mut self) {
            if self.tls_tp != 0 {
                self.x[4] = self.tls_tp;
            }
        }

        fn program_load(&mut self, prog_filename: &str) -> U64 {
            let (prog, entry, tp) =
                ElfVM::load_elf(prog_filename, self.max_prog_size).expect("Failed to load ELF");
            self.tls_tp = tp;
            self.program = prog;
            self.reset();
            entry
        }
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
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
    #[derive(Clone, Copy)]
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

use crate::tiny_riscv64::VM;

fn main() {
    let mut vm = tiny_riscv64::ElfVM::new(4096, 1024 * 1024);
    let entry_point = vm.program_load("example.elf");
    println!("Program entry point is at: 0x{:x}", entry_point);
}