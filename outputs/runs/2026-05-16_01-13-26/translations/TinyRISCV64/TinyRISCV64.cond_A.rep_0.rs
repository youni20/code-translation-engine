use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::sync::atomic::{AtomicBool, Ordering};
use std::{mem, slice};

pub struct VM {
    pc: u64,
    inst: u32,
    program: Vec<u8>,
    x: [u64; 32],
    stack: Vec<u8>,
    data: Vec<u8>,
    halted: AtomicBool,
    max_prog_size: usize,
    p_beg: u64,
    p_end: u64,
    d_beg: u64,
    d_end: u64,
    s_beg: u64,
    s_end: u64,
}

impl VM {
    pub fn new(stack_size: usize, max_program_size: usize) -> Self {
        let stack = vec![0u8; stack_size];
        let mut vm = VM {
            pc: 0,
            inst: 0,
            program: Vec::new(),
            x: [0; 32],
            stack,
            data: Vec::new(),
            halted: AtomicBool::new(false),
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

    pub fn program_load(&mut self, prog_filename: &str) -> io::Result<u64> {
        self.program = VM::load_program(prog_filename, self.max_prog_size)?;
        self.reset();
        Ok(self.p_beg)
    }

    pub fn program_load_from_memory(&mut self, prog: &[u8]) -> Result<u64, String> {
        if prog.len() > self.max_prog_size {
            return Err(format!("Program too large (max {} bytes)", self.max_prog_size));
        }
        self.program = prog.to_vec();
        self.reset();
        Ok(self.p_beg)
    }

    pub fn map_data_mem(&mut self, mem: &[u8]) -> u64 {
        self.data = mem.to_vec();
        self.reset();
        self.d_beg
    }

    pub fn register_set(&mut self, reg: usize, value: u64) -> Result<(), String> {
        if reg >= 32 {
            return Err("Invalid register number".to_string());
        }
        if reg != 0 {
            self.x[reg] = value;
        }
        Ok(())
    }

    pub fn register_get(&self, reg: usize) -> Result<u64, String> {
        if reg >= 32 {
            return Err("Invalid register number".to_string());
        }
        Ok(self.x[reg])
    }

    pub fn stack_push<T: Copy>(&mut self, val: T) -> u64 {
        let size = mem::size_of::<T>();
        self.x[2] -= size as u64;
        self.mem_store(self.x[2], &val);
        self.x[2]
    }

    pub fn stack_pop<T: Copy>(&mut self) -> T {
        let size = mem::size_of::<T>();
        let val = self.mem_load::<T>(self.x[2]);
        self.x[2] += size as u64;
        val
    }

    pub fn stack_peek<T: Copy>(&self) -> T {
        self.mem_load::<T>(self.x[2])
    }

    pub fn execute_program(&mut self, entry_point: u64, max_instructions: usize) -> Result<(), String> {
        let prog_sz = self.program.len();
        let sentinel_pc = (prog_sz + 3) & !3;

        self.pc = entry_point;
        self.halted.store(false, Ordering::Relaxed);
        let mut count = 0;

        if prog_sz < 4 {
            return Err("Program too small (must be at least 4 bytes)".to_string());
        }

        while !self.halted.load(Ordering::Relaxed) {
            if self.pc as usize > prog_sz - 4 {
                return Err("PC jumped program region".to_string());
            }
            if count > max_instructions {
                return Err("Maximum instruction count exceeded".to_string());
            }
            count += 1;

            self.execute_instruction()?;

            if self.pc == sentinel_pc as u64 {
                self.halted.store(true, Ordering::Relaxed);
            }
        }

        Ok(())
    }

    pub fn halt_program(&self) -> bool {
        let already_halted = self.halted.swap(true, Ordering::Relaxed);
        !already_halted
    }

    pub fn reset(&mut self) {
        self.x.fill(0);
        self.x[1] = (self.program.len() as u64 + 3) & !3;
        self.x[2] = (self.program.len() as u64) + 64 + (self.data.len() as u64) + 64 + (self.stack.len() as u64);
        self.x[8] = self.x[2];

        self.p_end = self.program.len() as u64;
        self.d_beg = self.program.len() as u64 + 64;
        self.d_end = self.program.len() as u64 + 64 + self.data.len() as u64;
        self.s_beg = self.program.len() as u64 + 64 + self.data.len() as u64 + 64;
        self.s_end = self.program.len() as u64 + 64 + self.data.len() as u64 + 64 + self.stack.len() as u64;
    }

    fn load_program(filename: &str, max_size: usize) -> io::Result<Vec<u8>> {
        let mut file = File::open(filename)?;
        let size = file.seek(SeekFrom::End(0))? as usize;
        if size > max_size {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("Program too large (max {})", max_size)));
        }
        file.seek(SeekFrom::Start(0))?;
        let mut prog = vec![0; size];
        file.read_exact(&mut prog)?;
        Ok(prog)
    }

    fn execute_instruction(&mut self) -> Result<(), String> {
        self.inst = self.mem_load::<u32>(self.pc);
        self.pc += 4;

        self.x[0] = 0;

        match self.opcode() {
            0x37 => self.x[self.rd() as usize] = self.imm_u(),
            0x17 => self.x[self.rd() as usize] = (self.pc - 4) + self.imm_u(),
            0x6F => {
                self.x[self.rd() as usize] = self.pc;
                self.pc = self.pc.wrapping_add(self.imm_j() as u64).wrapping_sub(4);
            }
            0x67 => {
                let target = (self.x[self.rs1() as usize].wrapping_add(self.imm_i() as u64)) & !1;
                self.x[self.rd() as usize] = self.pc;
                self.pc = target;
            }
            0x63 => self.exec_branch(self.funct3(), self.rs1(), self.rs2(), self.imm_b()),
            0x03 => self.exec_load(self.funct3(), self.rd(), self.rs1(), self.imm_i()),
            0x23 => self.exec_store(self.funct3(), self.rs1(), self.rs2(), self.imm_s()),
            0x13 => self.exec_alu_imm(self.funct3(), self.rd(), self.rs1(), self.imm_i()),
            0x1B => self.exec_alu_imm32(self.funct3(), self.rd(), self.rs1(), self.imm_i() as i32),
            0x33 => self.exec_alu_reg(self.funct3(), self.funct7(), self.rd(), self.rs1(), self.rs2()),
            0x3B => self.exec_alu_reg32(self.funct3(), self.funct7(), self.rd(), self.rs1(), self.rs2()),
            0x0F => (),
            0x73 => self.exec_system(self.funct3(), self.rd()),
            _ => return Err("Unknown opcode".to_string()),
        }

        Ok(())
    }

    fn mem_ptr<T>(&self, addr: u64) -> &[u8] {
        let addr_max = addr + mem::size_of::<T>() as u64 - 1;
        if addr_max < self.p_end {
            &self.program[addr as usize..=addr_max as usize]
        } else if addr >= self.d_beg && addr_max < self.d_end {
            &self.data[(addr - self.d_beg) as usize..=(addr_max - self.d_beg) as usize]
        } else if addr >= self.s_beg && addr_max < self.s_end {
            &self.stack[(addr - self.s_beg) as usize..=(addr_max - self.s_beg) as usize]
        } else {
            panic!("Memory access out of bounds");
        }
    }

    fn mem_load<T: Copy>(&self, addr: u64) -> T {
        unsafe { 
            slice::from_raw_parts(self.mem_ptr::<T>(addr).as_ptr() as *const T, 1)[0]
        }
    }

    fn mem_store<T: Copy>(&mut self, addr: u64, value: &T) {
        let mut dest = self.mem_ptr::<T>(addr).to_vec();
        unsafe {
            (dest.as_mut_ptr() as *mut T).copy_from_nonoverlapping(value, 1);
        }
        let addr_max = addr + mem::size_of::<T>() as u64 - 1;
        if addr_max < self.p_end {
            self.program[addr as usize..=addr_max as usize].copy_from_slice(&dest);
        } else if addr >= self.d_beg && addr_max < self.d_end {
            self.data[(addr - self.d_beg) as usize..=(addr_max - self.d_beg) as usize].copy_from_slice(&dest);
        } else if addr >= self.s_beg && addr_max < self.s_end {
            self.stack[(addr - self.s_beg) as usize..=(addr_max - self.s_beg) as usize].copy_from_slice(&dest);
        } else {
            panic!("Memory access out of bounds");
        }
    }

    fn opcode(&self) -> u8 { (self.inst & 0x7f) as u8 }
    fn funct3(&self) -> u8 { ((self.inst >> 12) & 0x7) as u8 }
    fn funct7(&self) -> u8 { ((self.inst >> 25) & 0x7f) as u8 }
    fn rd(&self) -> u8 { ((self.inst >> 7) & 0x1f) as u8 }
    fn rs1(&self) -> u8 { ((self.inst >> 15) & 0x1f) as u8 }
    fn rs2(&self) -> u8 { ((self.inst >> 20) & 0x1f) as u8 }
    fn imm_i(&self) -> i64 { ((self.inst as i32) >> 20) as i64 }
    fn imm_s(&self) -> i64 { (self.imm_i() & !0x1f) | self.rd() as i64 }
    fn imm_b(&self) -> i64 {
        (((self.inst as i64 & 0x80000000) >> 19) |
        ((self.inst as i64 & 0x80) << 4) |
        ((self.inst as i64 >> 20) & 0x7E0) |
        ((self.inst as i64 >> 7) & 0x1E)) as i64
    }
    fn imm_j(&self) -> i64 {
        (((self.inst as i64 & 0x80000000) >> 11) |
        ((self.inst as i64 & 0xFF000)) |
        ((self.inst as i64 >> 9) & 0x800) |
        ((self.inst as i64 >> 20) & 0x7FE)) as i64
    }
    fn imm_u(&self) -> u64 { (self.inst & 0xfffff000) as u64 }

    fn exec_branch(&mut self, funct3: u8, rs1: u8, rs2: u8, imm: i64) {
        let taken = match funct3 {
            0 => self.x[rs1 as usize] == self.x[rs2 as usize],
            1 => self.x[rs1 as usize] != self.x[rs2 as usize],
            4 => (self.x[rs1 as usize] as i64) < (self.x[rs2 as usize] as i64),
            5 => (self.x[rs1 as usize] as i64) >= (self.x[rs2 as usize] as i64),
            6 => self.x[rs1 as usize] < self.x[rs2 as usize],
            7 => self.x[rs1 as usize] >= self.x[rs2 as usize],
            _ => panic!("Unknown branch operation"),
        };
        if taken {
            self.pc = self.pc.wrapping_add(imm as u64).wrapping_sub(4);
        }
    }

    fn exec_load(&mut self, funct3: u8, rd: u8, rs1: u8, imm: i64) {
        let addr = self.x[rs1 as usize].wrapping_add(imm as u64);
        self.x[rd as usize] = match funct3 {
            0 => { self.mem_load::<i8>(addr) as i64 as u64 },
            1 => { self.mem_load::<i16>(addr) as i64 as u64 },
            2 => { self.mem_load::<i32>(addr) as i64 as u64 },
            3 => self.mem_load::<u64>(addr),
            4 => { self.mem_load::<u8>(addr) as u64 },
            5 => { self.mem_load::<u16>(addr) as u64 },
            6 => { self.mem_load::<u32>(addr) as u64 },
            _ => panic!("Unknown load operation"),
        };
    }

    fn exec_store(&mut self, funct3: u8, rs1: u8, rs2: u8, imm: i64) {
        let addr = self.x[rs1 as usize].wrapping_add(imm as u64);
        let value = self.x[rs2 as usize];
        match funct3 {
            0 => { self.mem_store::<u8>(addr, &(value as u8)); },
            1 => { self.mem_store::<u16>(addr, &(value as u16)); },
            2 => { self.mem_store::<u32>(addr, &(value as u32)); },
            3 => { self.mem_store::<u64>(addr, &value); },
            _ => panic!("Unknown store operation"),
        }
    }

    fn exec_alu_imm(&mut self, funct3: u8, rd: u8, rs1: u8, imm: i64) {
        self.x[rd as usize] = match funct3 {
            0 => { self.x[rs1 as usize].wrapping_add(imm as u64) },
            1 => { self.x[rs1 as usize] << (imm as u64 & 0x3f) },
            2 => { if (self.x[rs1 as usize] as i64) < imm { 1 } else { 0 } },
            3 => { if self.x[rs1 as usize] < imm as u64 { 1 } else { 0 } },
            4 => { self.x[rs1 as usize] ^ imm as u64 },
            5 => { 
                if (imm as u64) & 0x400 == 0 {
                    self.x[rs1 as usize] >> (imm as u64 & 0x3f)
                } else {
                    (self.x[rs1 as usize] as i64 >> (imm & 0x3f)) as u64
                }
            },
            6 => { self.x[rs1 as usize] | imm as u64 },
            7 => { self.x[rs1 as usize] & imm as u64 },
            _ => panic!("Unknown alu_imm operation"),
        }
    }

    fn exec_alu_imm32(&mut self, funct3: u8, rd: u8, rs1: u8, imm: i32) {
        let a = self.x[rs1 as usize] as u32;
        self.x[rd as usize] = match funct3 {
            0 => ((a.wrapping_add(imm as u32)) as i32) as i64 as u64,
            1 => ((a.wrapping_shl(imm as u32 & 0x1f)) as i32) as i64 as u64,
            5 => {
                if (imm & 0x400) == 0 {
                    ((a.wrapping_shr(imm as u32 & 0x1f)) as i32) as i64 as u64
                } else {
                    ((a as i32).wrapping_shr((imm & 0x1f) as u32)) as i64 as u64
                }
            }
            _ => panic!("Unknown alu_imm32 operation"),
        };
    }

    fn exec_alu_reg(&mut self, funct3: u8, funct7: u8, rd: u8, rs1: u8, rs2: u8) {
        let a = self.x[rs1 as usize];
        let b = self.x[rs2 as usize];
        self.x[rd as usize] = match (funct7 as u16) << 3 | funct3 as u16 {
            0x000 => a.wrapping_add(b),
            0x100 => a.wrapping_sub(b),
            0x001 => a.wrapping_shl(b as u32 & 0x3f),
            0x002 => if (a as i64) < (b as i64) { 1 } else { 0 },
            0x003 => if a < b { 1 } else { 0 },
            0x004 => a ^ b,
            0x005 => a.wrapping_shr(b as u32 & 0x3f),
            0x105 => ((a as i64).wrapping_shr(b as u32 & 0x3f)) as u64,
            0x006 => a | b,
            0x007 => a & b,

            0x008 => a.wrapping_mul(b),
            0x009 => self.mulh(a as i64, b as i64),
            0x00a => self.mulhsu(a as i64, b as u64),
            0x00b => self.mulhu(a, b),
            0x00c => {
                if b != 0 {
                    if a == i64::MIN as u64 && b == -1i64 as u64 {
                        i64::MIN as u64
                    } else {
                        (a as i64).wrapping_div(b as i64) as u64
                    }
                } else {
                    u64::MAX
                }
            }
            0x00d => {
                if b != 0 {
                    a.wrapping_div(b)
                } else {
                    u64::MAX
                }
            }
            0x00e => {
                if b != 0 {
                    if a == i64::MIN as u64 && b == -1i64 as u64 {
                        0
                    } else {
                        (a as i64).wrapping_rem(b as i64) as u64
                    }
                } else {
                    a
                }
            }
            0x00f => {
                if b != 0 {
                    a.wrapping_rem(b)
                } else {
                    a
                }
            }
            _ => panic!("Unknown alu_reg operation"),
        };
    }

    fn exec_alu_reg32(&mut self, funct3: u8, funct7: u8, rd: u8, rs1: u8, rs2: u8) {
        let a = self.x[rs1 as usize] as u32;
        let b = self.x[rs2 as usize] as u32;
        self.x[rd as usize] = match (funct7 as u16) << 3 | funct3 as u16 {
            0x000 => (a.wrapping_add(b) as i32) as i64 as u64,
            0x100 => (a.wrapping_sub(b) as i32) as i64 as u64,
            0x001 => (a.wrapping_shl(b & 0x1f) as i32) as i64 as u64,
            0x005 => (a.wrapping_shr(b & 0x1f) as i32) as i64 as u64,
            0x105 => (a.wrapping_shr(b & 0x1f) as i32) as i64 as u64,

            0x008 => (a.wrapping_mul(b) as i32) as i64 as u64,
            0x00c => ({
                if b != 0 {
                    if a as i32 == i32::MIN && b as i32 == -1 {
                        i32::MIN
                    } else {
                        (a as i32).wrapping_div(b as i32)
                    }
                } else {
                    -1
                }
            }) as i64 as u64,
            0x00d => ({
                if b != 0 {
                    a.wrapping_div(b) as i32
                } else {
                    -1i32
                }
            }) as i64 as u64,
            0x00e => ({
                if b != 0 {
                    (a as i32).wrapping_rem(b as i32)
                } else {
                    a as i32
                }
            }) as i64 as u64,
            0x00f => ({
                if b != 0 {
                    a.wrapping_rem(b) as i32
                } else {
                    a as i32
                }
            }) as i64 as u64,
            _ => panic!("Unknown alu_reg32 operation"),
        };
    }

    fn exec_system(&mut self, funct3: u8, _rd: u8) {
        if funct3 != 0 {
            self.handle_csr();
            return;
        }

        match self.inst {
            0x00000073 => self.handle_ecall(),
            0x00100073 => {
                let has_prev = self.pc >= 8 && self.mem_load::<u32>(self.pc - 8) == 0x01f01013;
                let has_next = (self.pc + 3 < self.p_end) && self.mem_load::<u32>(self.pc) == 0x40705013;
                if has_prev && has_next {
                    self.handle_semihost();
                } else {
                    self.halted.store(true, Ordering::Relaxed);
                }
            }
            0x10500073 | 0x30200073 | 0x10200073 | 0x00200073 => panic!(
                "Unsupported instruction at pc=0x{:x}: implement privilege handling",
                self.pc - 4
            ),
            _ => panic!("Unknown SYSTEM instruction 0x{:x} at pc=0x{:x}", self.inst, self.pc - 4),
        }
    }

    fn handle_csr(&self) {}

    fn handle_semihost(&self) {
        panic!("Semihosting not supported, implement handle_semihost()");
    }

    fn handle_ecall(&self) {
        panic!("ECALL not supported, implement handle_ecall()");
    }

    fn mulh(&self, a: i64, b: i64) -> u64 {
        (((a as i128) * (b as i128)) >> 64) as u64
    }

    fn mulhu(&self, a: u64, b: u64) -> u64 {
        (((a as u128) * (b as u128)) >> 64) as u64
    }

    fn mulhsu(&self, a: i64, b: u64) -> u64 {
        (((a as i128) * (b as i128)) >> 64) as u64
    }
}

fn main() {}