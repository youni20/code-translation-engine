use std::fs::File;
use std::io::{Read, Error};
use std::sync::atomic::{AtomicBool, Ordering};
use std::vec::Vec;
use std::convert::TryInto;
use std::sync::Arc;

mod tinyriscv64 {
    pub type U8 = u8;
    pub type U16 = u16;
    pub type U32 = u32;
    pub type U64 = u64;
    pub type I8 = i8;
    pub type I16 = i16;
    pub type I32 = i32;
    pub type I64 = i64;
}

use tinyriscv64::*;

const REGISTER_COUNT: usize = 32;

pub struct VM {
    pc: U64,
    inst: U32,
    program: Vec<U8>,
    x: [U64; REGISTER_COUNT],
    stack: Vec<U8>,
    data: Vec<U8>,
    halted: Arc<AtomicBool>,
    max_prog_size: usize,

    p_beg: U64,
    p_end: U64,
    d_beg: U64,
    d_end: U64,
    s_beg: U64,
    s_end: U64,
}

impl VM {
    pub fn new(stack_size: usize, max_program_size: usize) -> Self {
        let mut vm = VM { // Made vm mutable
            pc: 0,
            inst: 0,
            program: Vec::new(),
            x: [0; REGISTER_COUNT],
            stack: vec![0; stack_size],
            data: Vec::new(),
            halted: Arc::new(AtomicBool::new(false)),
            max_prog_size: max_program_size,

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

    pub fn program_load(&mut self, prog_filename: &str) -> Result<U64, Error> {
        self.program = self.load_program(prog_filename, self.max_prog_size)?;
        self.reset();
        Ok(self.p_beg)
    }

    pub fn load_bytecode(&mut self, prog: &[U8]) -> Result<U64, &'static str> {
        if prog.len() > self.max_prog_size {
            return Err("Program too large");
        }
        self.program = prog.to_vec();
        self.reset();
        Ok(self.p_beg)
    }

    pub fn map_data_mem(&mut self, mem: &[U8]) -> Result<U64, &'static str> {
        self.data = mem.into();
        self.reset();
        Ok(self.d_beg)
    }

    pub fn register_set(&mut self, reg: usize, value: U64) -> Result<(), &'static str> {
        if reg >= REGISTER_COUNT {
            return Err("Invalid register number");
        }
        if reg != 0 {
            self.x[reg] = value;
        }
        Ok(())
    }

    pub fn register_get(&self, reg: usize) -> Result<U64, &'static str> {
        if reg >= REGISTER_COUNT {
            return Err("Invalid register number");
        }
        Ok(self.x[reg])
    }

    pub fn stack_push<T: Sized + Copy>(&mut self, val: &T) -> U64 {
        self.x[2] -= std::mem::size_of::<T>() as u64;
        self.mem_store(self.x[2], val);
        self.x[2]
    }

    pub fn stack_pop<T: Sized + Copy>(&mut self) -> T {
        self.x[2] += std::mem::size_of::<T>() as u64;
        self.mem_load::<T>(self.x[2] - std::mem::size_of::<T>() as u64)
    }

    pub fn stack_peek<T: Sized + Copy>(&self) -> T {
        self.mem_load::<T>(self.x[2])
    }

    pub fn execute_program(&mut self, entry_point: U64, max_instructions: usize) -> Result<(), &'static str> {
        let prog_sz = self.program.len();
        let sentinel_pc = ((prog_sz + 3) & !3) as U64;

        if prog_sz < 4 {
            return Err("Program too small (must be at least 4 bytes)");
        }

        self.pc = entry_point;
        self.halted.store(false, Ordering::SeqCst);
        let mut count = 0;

        while !self.halted.load(Ordering::SeqCst) {
            if self.pc > (prog_sz - 4) as U64 {
                return Err("PC jumped program region");
            }
            if count >= max_instructions {
                return Err("Maximum instruction count exceeded");
            }

            self.execute_instruction();
            count += 1;

            if self.pc == sentinel_pc {
                self.halted.store(true, Ordering::SeqCst);
            }
        }
        Ok(())
    }

    pub fn halt_program(&self) -> bool {
        !self.halted.swap(true, Ordering::SeqCst)
    }

    fn reset(&mut self) {
        self.x.iter_mut().for_each(|xn| *xn = 0);
        self.x[1] = (self.program.len() + 3) as U64 & !3;
        self.x[2] = (self.program.len() + 64 + self.data.len() + 64 + self.stack.len()) as U64;
        self.x[8] = self.x[2];

        self.p_end = self.program.len() as U64;
        self.d_beg = self.program.len() as U64 + 64;
        self.d_end = self.program.len() as U64 + 64 + self.data.len() as U64;
        self.s_beg = self.program.len() as U64 + 64 + self.data.len() as U64 + 64;
        self.s_end = self.program.len() as U64 + 64 + self.data.len() as U64 + 64 + self.stack.len() as U64;
    }

    fn load_program(&self, filename: &str, max_size: usize) -> Result<Vec<U8>, Error> {
        let file = File::open(filename)?;
        let mut prog = Vec::new();
        file.take(max_size as u64).read_to_end(&mut prog)?;
        if prog.len() > max_size {
            return Err(Error::new(std::io::ErrorKind::InvalidData, "Program too large"));
        }
        Ok(prog)
    }

    fn opcode(&self) -> U8 {
        (self.inst & 0x7f) as U8
    }
    fn funct3(&self) -> U8 {
        ((self.inst >> 12) & 0x7) as U8
    }
    fn funct7(&self) -> U8 {
        ((self.inst >> 25) & 0x7f) as U8
    }
    fn rd(&self) -> U8 {
        ((self.inst >> 7) & 0x1f) as U8
    }
    fn rs1(&self) -> U8 {
        ((self.inst >> 15) & 0x1f) as U8
    }
    fn rs2(&self) -> U8 {
        ((self.inst >> 20) & 0x1f) as U8
    }
    fn imm_i(&self) -> I64 {
        (self.inst as I32 >> 20) as I64
    }
    fn imm_s(&self) -> I64 {
        (self.imm_i() & !0x1f) | self.rd() as I64
    }
    fn imm_b(&self) -> I64 {
        ((self.inst as u32 & 0x80000000u32) as I32 as I64 >> 19)
            | ((self.inst & 0x80) << 4) as I64
            | ((self.inst >> 20) & 0x7e0) as I64
            | ((self.inst >> 7) & 0x1e) as I64
    }
    fn imm_j(&self) -> I64 {
        ((self.inst as u32 & 0x80000000u32) as I32 as I64 >> 11)
            | ((self.inst & 0xff000) as I64)
            | ((self.inst >> 9) & 0x800) as I64
            | ((self.inst >> 20) & 0x7fe) as I64
    }
    fn imm_u(&self) -> U64 {
        (self.inst & 0xfffff000) as I64 as U64
    }

    fn execute_instruction(&mut self) {
        self.inst = u32::from_ne_bytes(self.program[self.pc as usize..self.pc as usize + 4].try_into().unwrap());
        self.pc += 4;

        self.x[0] = 0;

        match self.opcode() {
            0x37 => self.x[self.rd() as usize] = self.imm_u(),                       // LUI
            0x17 => self.x[self.rd() as usize] = (self.pc - 4) + self.imm_u(),       // AUIPC
            0x6f => {
                self.x[self.rd() as usize] = self.pc;
                self.pc = self.pc.wrapping_add(self.imm_j().try_into().unwrap()).wrapping_sub(4);        // JAL
            }
            0x67 => {
                let target = (self.x[self.rs1() as usize] + self.imm_i() as U64) & !1;
                self.x[self.rd() as usize] = self.pc;
                self.pc = target;                                                  // JALR
            }
            0x63 => self.exec_branch(self.funct3(), self.rs1(), self.rs2(), self.imm_b()),
            0x03 => self.exec_load(self.funct3(), self.rd(), self.rs1(), self.imm_i()),
            0x23 => self.exec_store(self.funct3(), self.rs1(), self.rs2(), self.imm_s()),
            0x13 => self.exec_alu_imm(self.funct3(), self.rd(), self.rs1(), self.imm_i()),
            0x1b => self.exec_alu_imm32(self.funct3(), self.rd(), self.rs1(), self.imm_i() as I32),
            0x33 => self.exec_alu_reg(self.funct3(), self.funct7(), self.rd(), self.rs1(), self.rs2()),
            0x3b => self.exec_alu_reg32(self.funct3(), self.funct7(), self.rd(), self.rs1(), self.rs2()),
            0x0f => {} // FENCE (nop)
            0x73 => self.exec_system(self.funct3(), self.rd()),
            _ => panic!("Unknown opcode"),
        }
    }

    fn mem_ptr<T>(&self, addr: U64) -> Result<*const u8, &'static str> {
        if addr > 0xFFFFFFFFFFFFFFF0 {
            return Err("Memory access out of bounds");
        }

        let addr_max = addr + (std::mem::size_of::<T>() - 1) as U64;

        if addr_max < self.p_end {
            return Ok(self.program.as_ptr().wrapping_add(addr as usize));
        } else if addr >= self.d_beg && addr_max < self.d_end {
            return Ok(self.data.as_ptr().wrapping_add((addr - self.d_beg) as usize));
        } else if addr >= self.s_beg && addr_max < self.s_end {
            return Ok(self.stack.as_ptr().wrapping_add((addr - self.s_beg) as usize));
        }

        Err("Memory access out of bounds")
    }

    fn mem_load<T>(&self, addr: U64) -> T
    where
        T: Copy,
    {
        let ptr: *const T = self.mem_ptr::<T>(addr).expect("Memory access out of bounds") as *const T;
        unsafe { *ptr }
    }

    fn mem_store<T: Copy>(&mut self, addr: U64, value: &T) {
        let ptr: *mut T = self.mem_ptr::<T>(addr).expect("Memory access out of bounds") as *mut T;
        unsafe { *ptr = *value };
    }

    fn exec_branch(&mut self, funct3: U8, rs1: U8, rs2: U8, imm: I64) {
        let taken;
        match funct3 {
            0 => taken = self.x[rs1 as usize] == self.x[rs2 as usize],             // BEQ
            1 => taken = self.x[rs1 as usize] != self.x[rs2 as usize],             // BNE
            4 => taken = (self.x[rs1 as usize] as I64) < (self.x[rs2 as usize] as I64), // BLT
            5 => taken = (self.x[rs1 as usize] as I64) >= (self.x[rs2 as usize] as I64), // BGE
            6 => taken = self.x[rs1 as usize] < self.x[rs2 as usize],              // BLTU
            7 => taken = self.x[rs1 as usize] >= self.x[rs2 as usize],             // BGEU
            _ => panic!("Unknown branch operation"),
        }
        if taken {
            self.pc = self.pc.wrapping_add((imm - 4).try_into().unwrap());
        }
    }

    fn exec_load(&mut self, funct3: U8, rd: U8, rs1: U8, imm: I64) {
        let addr = self.x[rs1 as usize].wrapping_add(imm as u64);
        match funct3 {
            0 => self.x[rd as usize] = self.mem_load::<I8>(addr) as U64, // LB
            1 => self.x[rd as usize] = self.mem_load::<I16>(addr) as U64, // LH
            2 => self.x[rd as usize] = self.mem_load::<I32>(addr) as U64, // LW
            3 => self.x[rd as usize] = self.mem_load::<U64>(addr), // LD
            4 => self.x[rd as usize] = self.mem_load::<U8>(addr) as U64,  // LBU
            5 => self.x[rd as usize] = self.mem_load::<U16>(addr) as U64, // LHU
            6 => self.x[rd as usize] = self.mem_load::<U32>(addr) as U64, // LWU
            _ => panic!("Unknown load operation"),
        }
    }

    fn exec_store(&mut self, funct3: U8, rs1: U8, rs2: U8, imm: I64) {
        let addr = self.x[rs1 as usize].wrapping_add(imm as u64);
        match funct3 {
            0 => self.mem_store(addr, &(self.x[rs2 as usize] as U8)),  // SB
            1 => self.mem_store(addr, &(self.x[rs2 as usize] as U16)), // SH
            2 => self.mem_store(addr, &(self.x[rs2 as usize] as U32)), // SW
            3 => {
                let value = self.x[rs2 as usize];
                self.mem_store(addr, &value);
            },          // SD
            _ => panic!("Unknown store operation"),
        }
    }

    fn exec_alu_imm(&mut self, funct3: U8, rd: U8, rs1: U8, imm: I64) {
        match funct3 {
            0 => self.x[rd as usize] = self.x[rs1 as usize].wrapping_add(imm as u64), // ADDI
            1 => self.x[rd as usize] = self.x[rs1 as usize] << (imm & 0x3f),         // SLLI
            2 => self.x[rd as usize] = ((self.x[rs1 as usize] as I64) < imm) as U64,   // SLTI
            3 => self.x[rd as usize] = (self.x[rs1 as usize] < imm as u64) as U64,   // SLTIU
            4 => self.x[rd as usize] = self.x[rs1 as usize] ^ imm as u64,          // XORI
            5 => {
                if imm & 0x400 == 0 {
                    self.x[rd as usize] = self.x[rs1 as usize] >> (imm & 0x3f) // SRLI
                } else {
                    self.x[rd as usize] = (self.x[rs1 as usize] as I64 >> (imm & 0x3f)) as u64 // SRAI
                }
            }
            6 => self.x[rd as usize] = self.x[rs1 as usize] | imm as u64, // ORI
            7 => self.x[rd as usize] = self.x[rs1 as usize] & imm as u64, // ANDI
            _ => panic!("Unknown alu_imm operation"),
        }
    }

    fn exec_alu_imm32(&mut self, funct3: U8, rd: U8, rs1: U8, imm: I32) {
        let result: U32;
        match funct3 {
            0 => result = (self.x[rs1 as usize] as U32).wrapping_add(imm as u32), // ADDIW
            1 => result = (self.x[rs1 as usize] as U32) << (imm & 0x1f), // SLLIW
            5 => {
                result = if (imm & 0x400) == 0 {
                    (self.x[rs1 as usize] as U32) >> (imm & 0x1f) // SRLIW
                } else {
                    (self.x[rs1 as usize] as I32 >> (imm & 0x1f)) as u32 // SRAIW
                }
            }
            _ => panic!("Unknown alu_imm32 operation"),
        }
        self.x[rd as usize] = result as I64 as U64; // Sign-extend
    }

    fn exec_alu_reg(&mut self, funct3: U8, funct7: U8, rd: U8, rs1: U8, rs2: U8) {
        let op = (funct7 as u16) << 3 | funct3 as u16;
        match op {
            0x000 => self.x[rd as usize] = self.x[rs1 as usize].wrapping_add(self.x[rs2 as usize]), // ADD
            0x001 => self.x[rd as usize] = self.x[rs1 as usize] << (self.x[rs2 as usize] & 0x3f), // SLL
            0x002 => {
                self.x[rd as usize] =
                    ((self.x[rs1 as usize] as I64) < (self.x[rs2 as usize] as I64)) as U64 // SLT
            }
            0x003 => self.x[rd as usize] = (self.x[rs1 as usize] < self.x[rs2 as usize]) as U64, // SLTU
            0x004 => self.x[rd as usize] = self.x[rs1 as usize] ^ self.x[rs2 as usize], // XOR
            0x005 => self.x[rd as usize] = self.x[rs1 as usize] >> (self.x[rs2 as usize] & 0x3f), // SRL
            0x105 => {
                self.x[rd as usize] =
                    (self.x[rs1 as usize] as I64 >> (self.x[rs2 as usize] & 0x3f)) as u64 // SRA
            }
            0x006 => self.x[rd as usize] = self.x[rs1 as usize] | self.x[rs2 as usize], // OR
            0x007 => self.x[rd as usize] = self.x[rs1 as usize] & self.x[rs2 as usize], // AND

            // M extension
            0x008 => self.x[rd as usize] = self.x[rs1 as usize].wrapping_mul(self.x[rs2 as usize]), // MUL
            0x009 => self.x[rd as usize] = dyn_mulh(self.x[rs1 as usize] as I64, self.x[rs2 as usize] as I64), // MULH
            0x00a => self.x[rd as usize] = dyn_mulhsu(self.x[rs1 as usize] as I64, self.x[rs2 as usize] as u64), // MULHSU
            0x00b => self.x[rd as usize] = dyn_mulhu(self.x[rs1 as usize], self.x[rs2 as usize]), // MULHU
            0x00c => { // DIV
                if self.x[rs2 as usize] != 0 {
                    self.x[rd as usize] = if self.x[rs1 as usize] as I64 == I64::MIN && self.x[rs2 as usize] as I64 == -1 {
                        I64::MIN as U64
                    } else {
                        (self.x[rs1 as usize] as I64 / self.x[rs2 as usize] as I64) as U64
                    }
                } else {
                    self.x[rd as usize] = u64::MAX
                }
            }
            0x00d => { // DIVU
                if self.x[rs2 as usize] != 0 {
                    self.x[rd as usize] = self.x[rs1 as usize] / self.x[rs2 as usize];
                } else {
                    self.x[rd as usize] = u64::MAX;
                }
            }
            0x00e => { // REM
                if self.x[rs2 as usize] != 0 {
                    self.x[rd as usize] = if self.x[rs1 as usize] as I64 == I64::MIN && self.x[rs2 as usize] as I64 == -1 {
                        0
                    } else {
                        (self.x[rs1 as usize] as I64 % self.x[rs2 as usize] as I64) as U64
                    }
                } else {
                    self.x[rd as usize] = self.x[rs1 as usize]
                }
            }
            0x00f => { // REMU
                if self.x[rs2 as usize] != 0 {
                    self.x[rd as usize] = self.x[rs1 as usize] % self.x[rs2 as usize];
                } else {
                    self.x[rd as usize] = self.x[rs1 as usize];
                }
            }
            _ => panic!("Unknown alu_reg operation"),
        }
    }

    fn exec_alu_reg32(&mut self, funct3: U8, funct7: U8, rd: U8, rs1: U8, rs2: U8) {
        let op = (funct7 as u16) << 3 | funct3 as u16;
        let a = self.x[rs1 as usize] as U32;
        let b = self.x[rs2 as usize] as U32;

        let result = match op {
            0x000 => a.wrapping_add(b) as I32,                             // ADDW
            0x101 => a.wrapping_sub(b) as I32,                             // SUBW
            0x001 => (a << (b & 0x1f)) as I32,                            // SLLW
            0x005 => (a >> (b & 0x1f)) as I32,                            // SRLW
            0x105 => ((a as I32 >> (b & 0x1f)) as u32) as I32,            // SRAW

            // M extension 32-bit
            0x008 => a.wrapping_mul(b) as I32,                             // MULW
            0x00c => {  // DIVW
                if b != 0 {
                    if a as I32 == I32::MIN && b as I32 == -1 {
                        I32::MIN
                    } else {
                        a as I32 / b as I32
                    }
                } else {
                    -1
                }
            },
            0x00d => {  // DIVUW
                if b != 0 {
                    (a / b) as I32
                } else {
                    -1
                }
            },
            0x00e => {  // REMW
                if b != 0 {
                    a as I32 % b as I32
                } else {
                    a as I32
                }
            },
            0x00f => {  // REMUW
                if b != 0 {
                    (a % b) as I32
                } else {
                    a as I32
                }
            },
            _ => panic!("Unknown alu_reg32 operation"),
        };

        self.x[rd as usize] = result as U64; // Sign-extend to 64 bits
    }

    fn exec_system(&mut self, funct3: U8, _rd: U8) {
        if funct3 != 0 {
            self.handle_csr();
            return;
        }

        match self.inst {
            0x00000073 => self.handle_ecall(),             // ECALL
            0x00100073 => {
                if self.check_semihosting() {
                    self.handle_semihost();
                } else {
                    self.halted.store(true, Ordering::SeqCst);
                }
            } // EBREAK
            0x10500073 | 0x10200073 | 0x00200073 => panic!("Privilege-mode return instruction at pc."),
            _ => panic!("Unknown SYSTEM instruction"),
        }
    }

    fn check_semihosting(&self) -> bool {
        let has_prev = (self.pc >= 8) && self.mem_load::<U32>(self.pc - 8) == 0x01f01013;
        let has_next = (self.pc + 3 < self.p_end) && self.mem_load::<U32>(self.pc) == 0x40705013;
        has_prev && has_next
    }

    fn handle_csr(&mut self) {
        let d = self.rd() as usize;
        if d != 0 {
            self.x[d] = 0;
        }
    }

    fn handle_semihost(&mut self) {
        panic!("Semihosting call not supported");
    }

    fn handle_ecall(&mut self) {
        panic!("ECALL not supported");
    }
}

fn dyn_mulh(a: I64, b: I64) -> U64 {
    let neg = (a < 0) ^ (b < 0);
    let abs_a = a.abs() as U64;
    let abs_b = b.abs() as U64;

    let (hi, lo) = mulu64_128(abs_a, abs_b);

    if neg {
        let (hi, lo) = (!hi, !lo);
        let lo = lo.wrapping_add(1);
        let hi = if lo == 0 { hi.wrapping_add(1) } else { hi };
        return hi as I64 as U64;
    }

    hi
}

fn dyn_mulhsu(a: I64, b: U64) -> U64 {
    if a < 0 {
        let abs_a = (-a) as U64;
        let (hi, lo) = mulu64_128(abs_a, b);
        let (hi, _) = handle_twos_complement(hi, lo);
        return hi as I64 as U64;
    }
    let (hi, _) = mulu64_128(a as U64, b);
    hi
}

fn dyn_mulhu(a: U64, b: U64) -> U64 {
    mulu64_128(a, b).0
}

fn handle_twos_complement(hi: U64, lo: U64) -> (U64, U64) {
    let (hi, lo) = (!hi, !lo);
    let lo = lo.wrapping_add(1);
    let hi = if lo == 0 { hi.wrapping_add(1) } else { hi };
    (hi, lo)
}

fn mulu64_128(a: U64, b: U64) -> (U64, U64) {
    const TRUNC32: U64 = 0xFFFFFFFF;

    let a_lo = a & TRUNC32;
    let a_hi = a >> 32;
    let b_lo = b & TRUNC32;
    let b_hi = b >> 32;

    let p0 = a_lo * b_lo;
    let p1 = a_lo * b_hi;
    let p2 = a_hi * b_lo;
    let p3 = a_hi * b_hi;

    let mid = (p0 >> 32) + (p1 & TRUNC32) + (p2 & TRUNC32);
    let lo = (p0 & TRUNC32) | (mid << 32);
    let hi = p3 + (p1 >> 32) + (p2 >> 32) + (mid >> 32);

    (hi, lo)
}

// No main function is provided for the VM.
fn main() {}