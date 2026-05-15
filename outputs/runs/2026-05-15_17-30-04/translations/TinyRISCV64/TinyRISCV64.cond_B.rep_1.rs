use std::fs::File;
use std::io::{self, Read};
use std::sync::atomic::{AtomicBool, Ordering};
use std::path::Path;

pub struct VM {
    pc: u64,                       // Program counter
    inst: u32,                     // Current instruction
    program: Vec<u8>,              // Program memory
    x: [u64; 32],                  // Registers x0-x31
    stack: Vec<u8>,                // Stack memory
    data: Vec<u8>,                 // Data memory
    halted: AtomicBool,            // Program exited or externally halted
    max_prog_size: usize,          // Maximum allowed program image size
    p_beg: u64,                    // Program mem begin
    p_end: u64,                    // Program mem end
    d_beg: u64,                    // Data mem begin
    d_end: u64,                    // Data mem end
    s_beg: u64,                    // Stack mem begin
    s_end: u64,                    // Stack mem end
}

impl VM {
    pub fn new(stack_size: usize, max_prog_size: usize) -> Self {
        let mut vm = VM {
            pc: 0,
            inst: 0,
            program: Vec::new(),
            x: [0; 32],
            stack: vec![0; stack_size],
            data: Vec::new(),
            halted: AtomicBool::new(false),
            max_prog_size,
            p_beg: 0,
            p_end: 0,
            d_beg: 0,
            d_end: 0,
            s_beg: 0,
            s_end: 0,
        };
        vm.reset();
        vm
    }

    pub fn program_load(&mut self, prog_filename: &str) -> io::Result<u64> {
        self.program = Self::load_program(prog_filename, self.max_prog_size)?;
        self.reset();
        Ok(self.p_beg)
    }

    pub fn program_load_from_memory(&mut self, prog: &[u8]) -> Result<u64, &'static str> {
        if prog.len() > self.max_prog_size {
            return Err("Program too large");
        }
        self.program.resize(prog.len(), 0);
        self.program.copy_from_slice(prog);
        self.reset();
        Ok(self.p_beg)
    }

    pub fn map_data_mem(&mut self, mem: &[u8]) -> u64 {
        self.data = mem.to_vec();
        self.reset();
        self.d_beg
    }

    pub fn register_set(&mut self, reg: usize, value: u64) -> Result<(), &'static str> {
        if reg >= 32 {
            return Err("Invalid register number");
        }
        if reg != 0 {
            self.x[reg] = value;
        }
        Ok(())
    }

    pub fn register_get(&self, reg: usize) -> Result<u64, &'static str> {
        if reg >= 32 {
            return Err("Invalid register number");
        }
        Ok(self.x[reg])
    }

    pub fn stack_push<T: Copy>(&mut self, val: T) -> u64 {
        let size = std::mem::size_of::<T>();
        self.x[2] -= size as u64;
        self.mem_store(self.x[2], val);
        self.x[2]
    }

    pub fn stack_pop<T: Copy>(&mut self) -> T {
        let size = std::mem::size_of::<T>();
        self.x[2] += size as u64;
        self.mem_load::<T>(self.x[2] - size as u64)
    }

    pub fn stack_peek<T: Copy>(&self) -> T {
        self.mem_load::<T>(self.x[2])
    }

    pub fn execute_program(&mut self, entry_point: u64, max_instructions: usize) -> Result<(), &'static str> {
        let prog_sz = self.program.len();
        let sentinel_pc = ((prog_sz + 3) & !3) as u64;

        self.pc = entry_point;
        self.halted.store(false, Ordering::SeqCst);
        let mut count = 0;

        if prog_sz < 4 {
            return Err("Program too small (must be at least 4 bytes)");
        }

        while !self.halted.load(Ordering::SeqCst) {
            if self.pc as usize > prog_sz - 4 {
                return Err("PC jumped program region");
            }
            if count >= max_instructions {
                return Err("Maximum instruction count exceeded");
            }

            self.execute_instruction();

            if self.pc == sentinel_pc {
                self.halted.store(true, Ordering::SeqCst);
            }

            count += 1;
        }
        Ok(())
    }

    pub fn halt_program(&self) -> bool {
        !self.halted.swap(true, Ordering::SeqCst)
    }

    pub fn reset(&mut self) {
        for xn in &mut self.x {
            *xn = 0;
        }
        //x1 - return address (ra)
        self.x[1] = (self.program.len() + 3) as u64 & !3;
        //x2 - stack pointer (sp)
        self.x[2] = self.program.len() as u64 + 64 + self.data.len() as u64 + 64 + self.stack.len() as u64;
        //x8 - frame pointer (s0 / fp)
        self.x[8] = self.x[2];

        self.p_end = self.program.len() as u64;
        /* 64 overflow detection addresses */
        self.d_beg = self.program.len() as u64 + 64;
        self.d_end = self.program.len() as u64 + 64 + self.data.len() as u64;
        /* 64 overflow detection addresses */
        self.s_beg = self.program.len() as u64 + 64 + self.data.len() as u64 + 64;
        self.s_end = self.program.len() as u64 + 64 + self.data.len() as u64 + 64 + self.stack.len() as u64;
    }

    fn load_program(filename: &str, max_size: usize) -> io::Result<Vec<u8>> {
        let mut file = File::open(Path::new(filename))?;
        let metadata = file.metadata()?;
        let size = metadata.len() as usize;

        if size > max_size {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Program too large"));
        }

        let mut prog = vec![0; size];
        file.read(&mut prog)?;
        Ok(prog)
    }

    fn opcode(&self) -> u8 {
        (self.inst & 0x7f) as u8
    }

    fn funct3(&self) -> u8 {
        ((self.inst >> 12) & 0x7) as u8
    }

    fn funct7(&self) -> u8 {
        ((self.inst >> 25) & 0x7f) as u8
    }

    fn rd(&self) -> u8 {
        ((self.inst >> 7) & 0x1f) as u8
    }

    fn rs1(&self) -> u8 {
        ((self.inst >> 15) & 0x1f) as u8
    }

    fn rs2(&self) -> u8 {
        ((self.inst >> 20) & 0x1f) as u8
    }

    fn imm_i(&self) -> i64 {
        (self.inst as i32 >> 20) as i64
    }

    fn imm_s(&self) -> i64 {
        (self.imm_i() & !0x1f) | self.rd() as i64
    }

    fn imm_b(&self) -> i64 {
        ((self.inst as i32 & 0x80000000) as i64 >> 19)
            | ((self.inst & 0x80) as i64 << 4)
            | (((self.inst >> 20) & 0x7e0) as i64)
            | (((self.inst >> 7) & 0x1e) as i64)
    }

    fn imm_j(&self) -> i64 {
        ((self.inst as i32 & 0x80000000) as i64 >> 11)
            | ((self.inst & 0xff000) as i64)
            | (((self.inst >> 9) & 0x800) as i64)
            | (((self.inst >> 20) & 0x7fe) as i64)
    }

    fn imm_u(&self) -> u64 {
        (self.inst & 0xfffff000) as u64
    }

    fn execute_instruction(&mut self) {
        self.inst = u32::from_le_bytes(self.program[self.pc as usize..self.pc as usize + 4].try_into().unwrap());
        self.pc += 4;

        self.x[0] = 0; // Ensure x0 stays zero

        match self.opcode() {
            0x37 => self.x[self.rd() as usize] = self.imm_u(),
            0x17 => self.x[self.rd() as usize] = self.pc - 4 + self.imm_u(),
            0x6f => {
                self.x[self.rd() as usize] = self.pc;
                self.pc = (self.pc as i64 + self.imm_j() - 4) as u64;
            }
            0x67 => {
                let target = (self.x[self.rs1() as usize] as i64 + self.imm_i()) as u64 & !1;
                self.x[self.rd() as usize] = self.pc;
                self.pc = target;
            }
            0x63 => {
                self.exec_branch(self.funct3(), self.rs1(), self.rs2(), self.imm_b());
            }
            0x03 => {
                self.exec_load(self.funct3(), self.rd(), self.rs1(), self.imm_i());
            }
            0x23 => {
                self.exec_store(self.funct3(), self.rs1(), self.rs2(), self.imm_s());
            }
            0x13 => {
                self.exec_alu_imm(self.funct3(), self.rd(), self.rs1(), self.imm_i());
            }
            0x1b => {
                self.exec_alu_imm32(self.funct3(), self.rd(), self.rs1(), self.imm_i() as i32);
            }
            0x33 => {
                self.exec_alu_reg(self.funct3(), self.funct7(), self.rd(), self.rs1(), self.rs2());
            }
            0x3b => {
                self.exec_alu_reg32(self.funct3(), self.funct7(), self.rd(), self.rs1(), self.rs2());
            }
            0x0f => {} // FENCE (nop)
            0x73 => {
                self.exec_system(self.funct3(), self.rd());
            }
            _ => panic!("Unknown opcode"),
        }
    }

    fn mem_ptr<T>(&self, addr: u64) -> *const u8 {
        if addr > 0xFFFFFFFFFFFFFFF0 {
            panic!("Memory access out of bounds");
        }

        let addr_max = addr + std::mem::size_of::<T>() as u64 - 1;

        if addr_max < self.p_end {
            return self.program.as_ptr().wrapping_add(addr as usize);
        }
        
        if addr >= self.d_beg && addr_max < self.d_end {
            return self.data.as_ptr().wrapping_add(addr as usize - self.d_beg as usize);
        }
        
        if addr >= self.s_beg && addr_max < self.s_end {
            return self.stack.as_ptr().wrapping_add(addr as usize - self.s_beg as usize);
        }

        panic!("Memory access out of bounds");
    }

    fn mem_load<T: Copy>(&self, addr: u64) -> T {
        unsafe { std::ptr::read(self.mem_ptr::<T>(addr) as *const T) }
    }

    fn mem_store<T: Copy>(&mut self, addr: u64, value: T) {
        unsafe { std::ptr::write(self.mem_ptr::<T>(addr) as *mut T, value) }
    }

    fn exec_branch(&mut self, funct3: u8, rs1: u8, rs2: u8, imm: i64) {
        let taken = match funct3 {
            0 => self.x[rs1 as usize] == self.x[rs2 as usize],               // BEQ
            1 => self.x[rs1 as usize] != self.x[rs2 as usize],               // BNE
            4 => (self.x[rs1 as usize] as i64) < (self.x[rs2 as usize] as i64),  // BLT
            5 => (self.x[rs1 as usize] as i64) >= (self.x[rs2 as usize] as i64), // BGE
            6 => self.x[rs1 as usize] < self.x[rs2 as usize],                   // BLTU
            7 => self.x[rs1 as usize] >= self.x[rs2 as usize],                  // BGEU
            _ => panic!("Unknown branch operation"),
        };
        if taken {
            self.pc = (self.pc as i64 + imm - 4) as u64;
        }
    }

    fn exec_load(&mut self, funct3: u8, rd: u8, rs1: u8, imm: i64) {
        let addr = (self.x[rs1 as usize] as i64 + imm) as u64;
        self.x[rd as usize] = match funct3 {
            0 => self.mem_load::<i8>(addr) as i64 as u64,   // LB
            1 => self.mem_load::<i16>(addr) as i64 as u64,  // LH
            2 => self.mem_load::<i32>(addr) as i64 as u64,  // LW
            3 => self.mem_load::<u64>(addr),                // LD
            4 => self.mem_load::<u8>(addr) as u64,          // LBU
            5 => self.mem_load::<u16>(addr) as u64,         // LHU
            6 => self.mem_load::<u32>(addr) as u64,         // LWU
            _ => panic!("Unknown load operation"),
        };
    }

    fn exec_store(&mut self, funct3: u8, rs1: u8, rs2: u8, imm: i64) {
        let addr = (self.x[rs1 as usize] as i64 + imm) as u64;
        match funct3 {
            0 => self.mem_store(addr, self.x[rs2 as usize] as u8),  // SB
            1 => self.mem_store(addr, self.x[rs2 as usize] as u16), // SH
            2 => self.mem_store(addr, self.x[rs2 as usize] as u32), // SW
            3 => self.mem_store(addr, self.x[rs2 as usize]),        // SD
            _ => panic!("Unknown store operation"),
        }
    }

    fn exec_alu_imm(&mut self, funct3: u8, rd: u8, rs1: u8, imm: i64) {
        self.x[rd as usize] = match funct3 {
            0 => self.x[rs1 as usize].wrapping_add(imm as u64), // ADDI
            1 => self.x[rs1 as usize] << (imm & 0x3f), // SLLI
            2 => if (self.x[rs1 as usize] as i64) < imm { 1 } else { 0 }, // SLTI
            3 => if self.x[rs1 as usize] < imm as u64 { 1 } else { 0 }, // SLTIU
            4 => self.x[rs1 as usize] ^ imm as u64, // XORI
            5 => {
                if (imm & 0x400) == 0 {
                    self.x[rs1 as usize] >> (imm & 0x3f) // SRLI
                } else {
                    (self.x[rs1 as usize] as i64 >> (imm & 0x3f)) as u64 // SRAI
                }
            }
            6 => self.x[rs1 as usize] | imm as u64, // ORI
            7 => self.x[rs1 as usize] & imm as u64, // ANDI
            _ => panic!("Unknown alu_imm operation"),
        };
    }

    fn exec_alu_imm32(&mut self, funct3: u8, rd: u8, rs1: u8, imm: i32) {
        let result = match funct3 {
            0 => (self.x[rs1 as usize] as u32).wrapping_add(imm as u32) as i32, // ADDIW
            1 => (self.x[rs1 as usize] as u32) << (imm & 0x1f), // SLLIW
            5 => {
                if (imm & 0x400) == 0 {
                    (self.x[rs1 as usize] as u32 >> (imm & 0x1f)) as i32 // SRLIW
                } else {
                    (self.x[rs1 as usize] as i32 >> (imm & 0x1f)) // SRAIW
                }
            }
            _ => panic!("Unknown alu_imm32 operation"),
        };
        self.x[rd as usize] = result as i64; // Sign-extend
    }

    fn exec_alu_reg(&mut self, funct3: u8, funct7: u8, rd: u8, rs1: u8, rs2: u8) {
        let op = funct7 << 3 | funct3;
        self.x[rd as usize] = match op {
            0x00 => self.x[rs1 as usize].wrapping_add(self.x[rs2 as usize]), // ADD
            0x01 => self.x[rs1 as usize] << (self.x[rs2 as usize] & 0x3f), // SLL
            0x02 => if (self.x[rs1 as usize] as i64) < (self.x[rs2 as usize] as i64) { 1 } else { 0 }, // SLT
            0x03 => if self.x[rs1 as usize] < self.x[rs2 as usize] { 1 } else { 0 }, // SLTU
            0x04 => self.x[rs1 as usize] ^ self.x[rs2 as usize], // XOR
            0x05 => self.x[rs1 as usize] >> (self.x[rs2 as usize] & 0x3f), // SRL
            0x06 => self.x[rs1 as usize] | self.x[rs2 as usize], // OR
            0x07 => self.x[rs1 as usize] & self.x[rs2 as usize], // AND
            0x08 => self.x[rs1 as usize].wrapping_mul(self.x[rs2 as usize]), // MUL
            0x09 => self.mulh(self.x[rs1 as usize] as i64, self.x[rs2 as usize] as i64), // MULH
            0x0a => self.mulhsu(self.x[rs1 as usize] as i64, self.x[rs2 as usize]), // MULHSU
            0x0b => self.mulhu(self.x[rs1 as usize], self.x[rs2 as usize]), // MULHU
            0x0c => {
                if self.x[rs2 as usize] != 0 {
                    if self.x[rs1 as usize] == i64::MIN as u64 && self.x[rs2 as usize] as i64 == -1 {
                        i64::MIN as u64
                    } else {
                        (self.x[rs1 as usize] as i64 / self.x[rs2 as usize] as i64) as u64
                    }
                } else {
                    u64::MAX
                }
            } // DIV
            0x0d => {
                if self.x[rs2 as usize] != 0 {
                    self.x[rs1 as usize] / self.x[rs2 as usize]
                } else {
                    u64::MAX
                }
            } // DIVU
            0x0e => {
                if self.x[rs2 as usize] != 0 {
                    if self.x[rs1 as usize] == i64::MIN as u64 && self.x[rs2 as usize] as i64 == -1 {
                        0
                    } else {
                        (self.x[rs1 as usize] as i64 % self.x[rs2 as usize] as i64) as u64
                    }
                } else {
                    self.x[rs1 as usize]
                }
            } // REM
            0x0f => {
                if self.x[rs2 as usize] != 0 {
                    self.x[rs1 as usize] % self.x[rs2 as usize]
                } else {
                    self.x[rs1 as usize]
                }
            } // REMU
            _ => panic!("Unknown alu_reg operation"),
        };
    }

    fn exec_alu_reg32(&mut self, funct3: u8, funct7: u8, rd: u8, rs1: u8, rs2: u8) {
        let op = funct7 << 3 | funct3;
        let a = self.x[rs1 as usize] as u32;
        let b = self.x[rs2 as usize] as u32;
        let result = match op {
            0x00 => a.wrapping_add(b) as i32,          // ADDW
            0x01 => a.wrapping_shl(b & 0x1f) as i32,   // SLLW
            0x05 => a.wrapping_shr(b & 0x1f) as i32,   // SRLW
            0x08 => (a.wrapping_mul(b)) as i32,        // MULW
            0x0c => {
                if b != 0 {
                    if a == i32::MIN as u32 && b as i32 == -1 {
                        i32::MIN
                    } else {
                        (a as i32 / b as i32)
                    }
                } else {
                    -1
                }
            } // DIVW
            0x0d => {
                if b != 0 {
                    (a / b) as i32
                } else {
                    -1
                }
            } // DIVUW
            0x0e => {
                if b != 0 {
                    (a as i32 % b as i32)
                } else {
                    a as i32
                }
            } // REMW
            0x0f => {
                if b != 0 {
                    (a % b) as i32
                } else {
                    a as i32
                }
            } // REMUW
            _ => panic!("Unknown alu_reg32 operation"),
        };
        self.x[rd as usize] = result as i64;
    }

    fn exec_system(&mut self, funct3: u8, _rd: u8) {
        if funct3 != 0 {
            self.handle_csr();
            return;
        }

        match self.inst {
            0x00000073 => {
                // ECALL
                self.handle_ecall();
            }
            0x00100073 => {
                // EBREAK
                // Check for the semihosting magic bracket:
                //   slli zero,zero,0x1f  (0x01f01013)  <-- instruction before ebreak
                //   ebreak
                //   srai zero,zero,0x7   (0x40705013)  <-- instruction after ebreak
                // pc is already advanced past the ebreak at this point.
                let has_prev = self.pc >= 8 && self.mem_load::<u32>(self.pc - 8) == 0x01f01013;
                let has_next = self.pc + 3 < self.p_end && self.mem_load::<u32>(self.pc) == 0x40705013;
                if has_prev && has_next {
                    self.handle_semihost();
                } else {
                    self.halted.store(true, Ordering::SeqCst);
                }
            }
            0x10500073 => {
                // WFI  (wait for interrupt)
                panic!(
                    "WFI (wait-for-interrupt) is not supported in this VM; \
                     remove interrupt-driven idle loops from bare-metal code"
                );
            }
            0x30200073 | 0x10200073 | 0x00200073 => {
                // MRET, SRET, URET
                panic!(
                    "Privilege-mode return instruction (MRET/SRET/URET) at pc=0x{:x}: \
                     this VM has no privilege levels",
                    self.pc - 4
                );
            }
            _ => panic!(
                "Unknown SYSTEM instruction 0x{:x} at pc=0x{:x}",
                self.inst,
                self.pc - 4
            ),
        }
    }

    fn handle_csr(&mut self) {
        // Stub: All CSRs read as 0; write side-effects are ignored
        let d = self.rd();
        if d != 0 {
            self.x[d as usize] = 0;
        }
    }

    fn handle_semihost(&self) {
        panic!(
            "Semihosting call at pc=0x{:x} is not supported in this VM; \
             implement handle_semihost() to support semihosting operations",
            self.pc - 4
        );
    }

    fn handle_ecall(&self) {
        panic!(
            "ECALL at pc=0x{:x} is not supported in this VM; \
             implement handle_ecall() to support system calls",
            self.pc - 4
        );
    }

    #[cfg(target_pointer_width = "64")]
    fn mulh(&mut self, a: i64, b: i64) -> u64 {
        let neg = (a < 0) ^ (b < 0);
        let abs_a = if a < 0 { 0u64.wrapping_sub(a as u64) } else { a as u64 };
        let abs_b = if b < 0 { 0u64.wrapping_sub(b as u64) } else { b as u64 };

        let (hi, lo) = self.mulu64_128(abs_a, abs_b);

        if neg {
            let hi = !hi;
            let lo = !lo;
            let lo = lo.wrapping_add(1);

            let hi = if lo == 0 { hi.wrapping_add(1) } else { hi };

            hi as i64 as u64
        } else {
            hi
        }
    }

    #[cfg(target_pointer_width = "64")]
    fn mulhu(&mut self, a: u64, b: u64) -> u64 {
        self.mulu64_128(a, b).0
    }

    #[cfg(target_pointer_width = "64")]
    fn mulu64_128(&self, a: u64, b: u64) -> (u64, u64) {
        const TRUNC32: u64 = 0xFFFFFFFF;
        let a_lo = a & TRUNC32;
        let a_hi = a >> 32;
        let b_lo = b & TRUNC32;
        let b_hi = b >> 32;

        let p0 = a_lo.wrapping_mul(b_lo);
        let p1 = a_lo.wrapping_mul(b_hi);
        let p2 = a_hi.wrapping_mul(b_lo);
        let p3 = a_hi.wrapping_mul(b_hi);

        let mid = (p0 >> 32).wrapping_add(p1 & TRUNC32).wrapping_add(p2 & TRUNC32);
        let lo = (p0 & TRUNC32) | (mid << 32);
        let hi = p3.wrapping_add(p1 >> 32).wrapping_add(p2 >> 32).wrapping_add(mid >> 32);

        (hi, lo)
    }

    #[cfg(target_pointer_width = "64")]
    fn mulhsu(&self, a: i64, b: u64) -> u64 {
        if a < 0 {
            let abs_a = 0u64.wrapping_sub(a as u64);
            let (hi, lo) = self.mulu64_128(abs_a, b);

            // 2's complement
            let hi = !hi;
            let lo = !lo;
            let lo = lo.wrapping_add(1);

            if lo == 0 {
                (hi.wrapping_add(1)) as i64 as u64
            } else {
                hi as i64 as u64
            }
        } else {
            self.mulu64_128(a as u64, b).0
        }
    }
}

fn main() {}