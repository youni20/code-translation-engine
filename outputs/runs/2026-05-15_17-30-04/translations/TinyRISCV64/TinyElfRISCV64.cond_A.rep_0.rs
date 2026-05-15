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
    use std::io::{Read, Seek, SeekFrom, Write};
    use std::time::{SystemTime, UNIX_EPOCH};
    use std::vec::Vec;
    use std::rc::Rc;
    use std::cell::RefCell;

    type U64 = u64;
    type I64 = i64;
    type U32 = u32;
    type U16 = u16;
    type U8 = u8;

    struct VM {
        stack_size: usize,
        max_program_size: usize,
        pc: U64,
        x: [U64; 32],
        halted: bool,
    }

    impl VM {
        pub fn new(stack_size: usize, max_program_size: usize) -> Self {
            Self {
                stack_size,
                max_program_size,
                pc: 0,
                x: [0; 32],
                halted: false,
            }
        }

        pub fn reset(&mut self) {
            self.pc = 0;
            self.x = [0; 32];
            self.halted = false;
        }
    }

    struct ElfVM {
        base: VM,
        fd_streams: HashMap<U64, Rc<RefCell<dyn IStream>>>,
        tls_tp: U64,
        program: Vec<U8>,
    }

    trait IStream: Read + Write + Seek {}

    impl ElfVM {
        pub fn new(stack_size: usize, max_program_size: usize) -> Self {
            Self {
                base: VM::new(stack_size, max_program_size),
                fd_streams: HashMap::new(),
                tls_tp: 0,
                program: Vec::new(),
            }
        }

        pub fn program_load(&mut self, prog_filename: &str) -> Result<U64, String> {
            let (prog, entry, tp) = Self::load_elf(prog_filename, self.base.max_program_size)?;
            self.tls_tp = tp;
            self.program = prog;
            self.reset();
            Ok(entry)
        }

        pub fn reset(&mut self) {
            self.base.reset();
            if self.tls_tp != 0 {
                self.base.x[4] = self.tls_tp;
            }
        }

        pub fn map_fd(&mut self, fd: U64, stream: Rc<RefCell<dyn IStream>>) {
            self.fd_streams.insert(fd, stream);
        }

        fn mem_read_str(&self, addr: U64) -> String {
            let mut s = String::new();
            let mut address = addr;
            loop {
                let c = self.mem_load::<U8>(address) as char;
                if c == '\0' {
                    break;
                }
                s.push(c);
                address += 1;
            }
            s
        }

        fn handle_semihost(&mut self) {
            let op = self.base.x[10];
            let arg = self.base.x[11];
            let argv = |n: usize| self.mem_load::<U64>(arg + n as U64 * 8);

            match op {
                0x01 => { // SYS_OPEN(path_ptr, mode, path_len)
                    let path = self.mem_read_str(argv(0));
                    let mode = argv(1);

                    self.base.x[10] = if path == ":tt" {
                        match mode {
                            0..=3 => 0,        // stdin
                            4..=7 => 1,        // stdout
                            8..=11 => 2,       // stderr
                            _ => !0 // -1LL
                        }
                    } else {
                        !0 // -1LL
                    };

                    return;
                }

                0x02 => { // SYS_CLOSE(fd)
                    let fd = argv(0);
                    self.fd_streams.remove(&fd);
                    self.base.x[10] = 0;
                    return;
                }

                0x03 => { // SYS_WRITEC(char_ptr) Write one character to stdout
                    let c = self.mem_load::<U8>(arg) as char;
                    if let Some(writer) = self.fd_streams.get(&1) {
                        writer.borrow_mut().write_all(&[c as u8]).ok();
                    }
                    self.base.x[10] = 0;
                    return;
                }

                0x04 => { // SYS_WRITE0(str_ptr) Write null-terminated string to stdout
                    let s = self.mem_read_str(arg);
                    if let Some(writer) = self.fd_streams.get(&1) {
                        writer.borrow_mut().write_all(s.as_bytes()).ok();
                    }
                    self.base.x[10] = 0;
                    return;
                }

                0x05 => { // SYS_WRITE(fd, buf_ptr, len)
                    let fd = argv(0);
                    let buf = argv(1);
                    let mut len = argv(2);

                    if let Some(writer) = self.fd_streams.get(&fd) {
                        for i in 0..len {
                            let byte = self.mem_load::<U8>(buf + i);
                            if let Err(_) = writer.borrow_mut().write_all(&[byte]) {
                                break;
                            }
                            len -= 1;
                        }
                    }
                    self.base.x[10] = len;
                    return;
                }

                0x06 => { // SYS_READ(fd, buf_ptr, len)
                    let fd = argv(0);
                    let buf = argv(1);
                    let len = argv(2);

                    if let Some(reader) = self.fd_streams.get(&fd) {
                        let mut buffer = vec![0u8; len as usize];
                        let n = reader.borrow_mut().read(&mut buffer).unwrap_or(0) as U64;
                        for i in 0..n {
                            self.mem_store(buf + i, buffer[i as usize]);
                        }
                        self.base.x[10] = len - n;
                        return;
                    }
                    self.base.x[10] = len;
                    return;
                }

                0x07 => { // SYS_READC
                    if let Some(reader) = self.fd_streams.get(&0) {
                        let mut buf = [0u8; 1];
                        if reader.borrow_mut().read(&mut buf).is_ok() {
                            self.base.x[10] = buf[0] as U64;
                            return;
                        }
                    }
                    self.base.x[10] = 0xFFFFFFFFFFFFFF04; // -1LL & ASCII_EOT
                }

                0x09 => { // SYS_ISTTY(fd)
                    let fd = argv(0);
                    self.base.x[10] = if fd <= 2 { 1 } else { 0 };
                    return;
                }

                0x0a => { // SYS_SEEK(fd, offset)
                    let fd = argv(0);
                    let off = argv(1) as i64;
                    if let Some(stream) = self.fd_streams.get(&fd) {
                        let result = stream
                            .borrow_mut()
                            .seek(SeekFrom::Start(off as u64))
                            .is_ok();
                        self.base.x[10] = if result { 0 } else { !0 };
                        return;
                    }
                    self.base.x[10] = !0;
                    return;
                }

                0x0c => { // SYS_FLEN(fd)
                    let fd = argv(0);
                    if let Some(stream) = self.fd_streams.get(&fd) {
                        let current_pos = stream.borrow_mut().stream_position().unwrap();
                        let end_pos = stream.borrow_mut().seek(SeekFrom::End(0)).unwrap();
                        stream.borrow_mut().seek(SeekFrom::Start(current_pos)).unwrap();
                        self.base.x[10] = end_pos;
                        return;
                    }
                    self.base.x[10] = !0;
                    return;
                }

                0x11 => { // SYS_TIME
                    self.base.x[10] = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs() as u64;
                    return;
                }

                0x18 => { // SYS_EXIT
                    self.base.halted = true;
                    self.base.x[10] = 0;
                    return;
                }

                0x30 => { // SYS_EXIT_EXTENDED(reason, exit_code)
                    let exit_code = argv(1);
                    self.base.halted = true;
                    self.base.x[10] = exit_code; // propagate exit code
                    return;
                }

                _ => {
                    panic!(
                        "Unsupported semihosting operation 0x{:x} at pc=0x{:x}",
                        op, self.base.pc - 4
                    );
                }
            }
        }

        fn handle_ecall(&mut self) {
            let num = self.base.x[17];
            let a0 = self.base.x[10];
            let a1 = self.base.x[11];
            let a2 = self.base.x[12];

            match num {
                93 | 94 => {
                    // exit(status)
                    self.base.halted = true;
                    return;
                }

                9 | 215 | 222 | 214 | 226 | 233 | 216 | 219 | 228 => {
                    // MMU / paging operations are not supported
                    self.base.x[10] = -38i64 as u64; // -ENOSYS
                    return;
                }

                57 => { // close(fd)
                    self.fd_streams.remove(&a0);
                    self.base.x[10] = 0;
                    return;
                }

                62 => { // lseek(fd, offset, whence)
                    if let Some(stream) = self.fd_streams.get(&a0) {
                        if a2 < 3 {
                            let pos = stream.borrow_mut().seek(SeekFrom::Start(a1));
                            self.base.x[10] = pos.unwrap_or(-29i32 as u64);
                        } else {
                            self.base.x[10] = -22i64 as u64; // -EINVAL
                        }
                    } else {
                        self.base.x[10] = -9i64 as u64; // -EBADF
                    }
                    return;
                }

                63 => { // read(fd, buf, count)
                    if let Some(stream) = self.fd_streams.get(&a0) {
                        let mut buf = vec![0; a2 as usize];
                        let n = stream.borrow_mut().read(&mut buf).unwrap_or(0) as u64;
                        for i in 0..n {
                            self.mem_store(a1 + i, buf[i as usize]);
                        }
                        self.base.x[10] = n;
                    } else {
                        self.base.x[10] = -9i64 as u64; // -EBADF
                    }
                    return;
                }

                64 => { // write(fd, buf, count)
                    if let Some(writer) = self.fd_streams.get(&a0) {
                        let mut buf = vec![0; a2 as usize];
                        for i in 0..a2 {
                            buf[i as usize] = self.mem_load::<U8>(a1 + i) as u8;
                        }
                        writer.borrow_mut().write_all(&buf).ok();
                        self.base.x[10] = if writer.borrow_mut().flush().is_ok() {
                            a2
                        } else {
                            -5i64 as u64 // -EIO
                        };
                    } else {
                        self.base.x[10] = -9i64 as u64; // -EBADF
                    }
                    return;
                }

                56 | 65 | 66 | 79 | 80 => {
                    self.base.x[10] = -38i64 as u64; // -ENOSYS
                    return;
                }

                160 => {
                    self.base.x[10] = -38i64 as u64; // -ENOSYS
                    return;
                }

                113 | 169 => {
                    self.base.x[10] = -22i64 as u64; // -EINVAL
                    return;
                }

                174 | 175 | 176 | 177 => {
                    self.base.x[10] = 0;
                    return;
                }

                96 => {
                    self.base.x[10] = 1;
                    return;
                }

                99 | 100 | 261 | 132 | 134 | 135 => {
                    self.base.x[10] = 0;
                    return;
                }

                220 | 221 => {
                    self.base.x[10] = -38i64 as u64; // -ENOSYS
                    return;
                }

                _ => {
                    panic!(
                        "ecall: unsupported syscall number {} (a0={}, a1={:#x}, a2={}) at pc={:#x}",
                        num, a0, a1, a2, self.base.pc - 4
                    );
                }
            }
        }

        fn mem_load<T: Copy>(&self, _: U64) -> T {
            unimplemented!()
        }

        fn mem_store<T: Copy>(&self, _: U64, _: T) {
            unimplemented!()
        }

        fn load_elf(filename: &str, max_size: usize) -> Result<(Vec<U8>, U64, U64), String> {
            let mut fin = File::open(filename).map_err(|_| format!("Failed to open ELF file: {}", filename))?;
            let file_size = fin.metadata().map_err(|_| format!("Failed to read metadata for: {}", filename))?.len();
            if file_size < std::mem::size_of::<Elf64Ehdr>() as u64 {
                return Err(format!("File too small to be a valid ELF64 binary: {}", filename));
            }

            let mut file_data = vec![0; file_size as usize];
            fin.read_exact(&mut file_data).map_err(|_| "Failed to read ELF file")?;

            let ehdr: Elf64Ehdr = unsafe { std::ptr::read(file_data.as_ptr() as *const _) };
            if ehdr.e_ident[0] != 0x7f
                || ehdr.e_ident[1] != b'E'
                || ehdr.e_ident[2] != b'L'
                || ehdr.e_ident[3] != b'F'
            {
                return Err("Not an ELF file (bad magic number)".into());
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
                    "Not a RISC-V ELF (e_machine={:#x}); recompile targeting riscv64",
                    ehdr.e_machine
                ));
            }

            if ehdr.e_type == 3 {
                return Err("ELF is a shared object / position-independent executable; relink as a static executable with -static -no-pie".into());
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
                    "ELF uses a hardware floating-point ABI (e_flags={:#x}); this VM implements RV64IM (integer only). Recompile with -march=rv64im -mabi=lp64",
                    ehdr.e_flags
                ));
            }

            if ehdr.e_flags & EF_RISCV_RVC != 0 {
                return Err(format!(
                    "ELF contains RISC-V Compressed (C) extension instructions (EF_RISCV_RVC set in e_flags={:#x}); this VM only handles 32-bit instructions. Recompile with -march=rv64im (omit 'c' from the march string) or add -mno-rvc",
                    ehdr.e_flags
                ));
            }

            if ehdr.e_flags & EF_RISCV_RVE != 0 {
                return Err(format!(
                    "ELF uses the RV32E reduced (16-register) integer ABI (EF_RISCV_RVE in e_flags={:#x}); recompile targeting riscv64",
                    ehdr.e_flags
                ));
            }

            if ehdr.e_phoff == 0 || ehdr.e_phnum == 0 {
                return Err(
                    "ELF has no program headers; link as a static executable, not a relocatable object (.o)"
                        .into(),
                );
            }

            if ehdr.e_phentsize < std::mem::size_of::<Elf64Phdr>() as u16 {
                return Err(format!(
                    "ELF program header entry size too small (e_phentsize={}; expected >= {})",
                    ehdr.e_phentsize,
                    std::mem::size_of::<Elf64Phdr>()
                ));
            }

            let phtab_end = ehdr.e_phoff + ehdr.e_phnum as u64 * ehdr.e_phentsize as u64;
            if phtab_end > file_data.len() as u64 {
                return Err("ELF program header table extends beyond end of file".into());
            }

            let mut has_interp = false;
            let mut has_dynamic = false;
            let mut vaddr_min = !0u64;
            let mut vaddr_max = 0u64;
            let mut tls_tp = 0u64;

            for i in 0..ehdr.e_phnum {
                let phdr: Elf64Phdr = unsafe {
                    std::ptr::read(
                        file_data
                            .as_ptr()
                            .offset(ehdr.e_phoff as isize + i as isize * ehdr.e_phentsize as isize)
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
                                i, phdr.p_filesz, phdr.p_memsz,
                            ));
                        }

                        if phdr.p_offset + phdr.p_filesz > file_data.len() as u64 {
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
                return Err(
                    "ELF requires a dynamic linker (PT_INTERP segment present); recompile and link with -static"
                        .into(),
                );
            }

            if has_dynamic {
                return Err(
                    "ELF contains dynamic linking information (PT_DYNAMIC segment present); recompile and link with -static"
                        .into(),
                );
            }

            if vaddr_min == !0 {
                return Err(
                    "ELF has no loadable (PT_LOAD) segments — nothing to execute".into(),
                );
            }

            if vaddr_max > max_size as u64 {
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
                let phdr: Elf64Phdr = unsafe {
                    std::ptr::read(
                        file_data
                            .as_ptr()
                            .offset(ehdr.e_phoff as isize + i as isize * ehdr.e_phentsize as isize)
                            as *const _,
                    )
                };

                if phdr.p_type != 1 || phdr.p_filesz == 0 {
                    continue;
                }

                let start = phdr.p_vaddr as usize;
                let len = phdr.p_filesz as usize;
                let source_start = phdr.p_offset as usize;
                prog[start..start + len].copy_from_slice(
                    &file_data[source_start..source_start + len],
                );
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
    // Please fill in the `main` function with appropriate logic or testing code.
}