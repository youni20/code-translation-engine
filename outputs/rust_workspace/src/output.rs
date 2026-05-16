use std::fs::File;
use std::io::{self, Read};
use std::sync::atomic::{AtomicBool, Ordering};
use std::ptr;
use std::mem;

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
    pub fn new(stack_size: usize, max_program_size: usize) -> VM {
        VM {
            pc: 0,
            inst: 0,
            program: Vec::new(),
            x: [0; 32],
            stack: vec![0; stack_size],
            data: Vec::new(),
            halted: AtomicBool::new(false),
            max_prog_size: max_program_size,
            p_beg: 0,
            p_end: 0,
            d_beg: 0,
            d_end: 0,
            s_beg: 0,
            s_end: 0,
        }
    }

    pub fn program_load(&mut self, prog_filename: &str) -> Result<u64, io::Error> {
        self.program = Self::load_program(prog_filename, self.max_prog_size)?;
        self.reset();
        Ok(self.p_beg)
    }

    pub fn program_load_from_slice(&mut self, prog: &[u8]) -> Result<u64, &'static str> {
        if prog.len() > self.max_prog_size {
            return Err("Program too large");
        }
        self.program = Vec::from(prog);
        self.reset();
        Ok(self.p_beg)
    }

    pub fn map_data_mem(&mut self, mem: &[u8]) -> u64 {
        self.data = Vec::from(mem);
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

    pub fn stack_push<T>(&mut self, val: T) -> u64 {
        self.x[2] -= mem::size_of::<T>() as u64;
        self.mem_store(self.x[2], val);
        self.x[2]
    }

    pub fn stack_pop<T>(&mut self) -> T {
        self.x[2] += mem::size_of::<T>() as u64;
        self.mem_load::<T>(self.x[2] - mem::size_of::<T>() as u64)
    }

    pub fn stack_peek<T>(&self) -> T {
        self.mem_load::<T>(self.x[2])
    }

    pub fn execute_program(&mut self, entry_point: u64, max_instructions: usize) -> Result<(), &'static str> {
        let prog_sz = self.program.len() as u64;
        let sentinel_pc = (prog_sz + 3) & !3u64;

        self.pc = entry_point;
        self.halted.store(false, Ordering::Relaxed);
        let mut count = 0;

        if prog_sz < 4 {
            return Err("Program too small");
        }

        while !self.halted.load(Ordering::Relaxed) {
            if self.pc > prog_sz - 4 {
                return Err("PC jumped program region");
            }

            count += 1;
            if count > max_instructions {
                return Err("Maximum instruction count exceeded");
            }

            self.execute_instruction()?;

            if self.pc == sentinel_pc {
                self.halted.store(true, Ordering::Relaxed);
            }
        }

        Ok(())
    }

    pub fn halt_program(&self) -> bool {
        !self.halted.swap(true, Ordering::Relaxed)
    }

    pub fn reset(&mut self) {
        self.x.iter_mut().for_each(|xn| *xn = 0);
        self.x[1] = (self.program.len() as u64 + 3) & !3u64;
        self.x[2] = self.p_end + 64 + self.s_end + self.stack.len() as u64;
        self.x[8] = self.x[2];

        self.p_end = self.program.len() as u64;
        self.d_beg = self.program.len() as u64 + 64;
        self.d_end = self.d_beg + self.data.len() as u64;
        self.s_beg = self.d_end + 64;
        self.s_end = self.s_beg + self.stack.len() as u64;
    }

    fn load_program(filename: &str, max_size: usize) -> Result<Vec<u8>, io::Error> {
        let file = File::open(filename)?;
        let file_size = file.metadata()?.len() as usize;

        if file_size > max_size {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Program too large"));
        }

        let mut buffer = Vec::with_capacity(file_size);
        file.take(file_size as u64).read_to_end(&mut buffer)?;

        Ok(buffer)
    }

    fn execute_instruction(&mut self) -> Result<(), &'static str> {
        self.inst = self.mem_load::<u32>(self.pc);
        self.pc += 4;

        self.x[0] = 0;

        match self.opcode() {
            0x37 => self.x[self.rd() as usize] = self.imm_u(),
            0x17 => self.x[self.rd() as usize] = self.pc - 4 + self.imm_u(),
            0x6f => {
                self.x[self.rd() as usize] = self.pc;
                self.pc = (self.pc as i64 + self.imm_j() - 4) as u64;
            }
            0x67 => {
                let target = ((self.x[self.rs1() as usize] as i64 + self.imm_i()) & !1i64) as u64;
                self.x[self.rd() as usize] = self.pc;
                self.pc = target;
            }
            0x63 => {
                if self.exec_branch(self.funct3(), self.rs1(), self.rs2(), self.imm_b())? {
                    self.pc = (self.pc as i64 + self.imm_b() - 4) as u64;
                }
            }
            0x03 => self.exec_load(self.funct3(), self.rd(), self.rs1(), self.imm_i())?,
            0x23 => self.exec_store(self.funct3(), self.rs1(), self.rs2(), self.imm_s())?,
            0x13 => self.exec_alu_imm(self.funct3(), self.rd(), self.rs1(), self.imm_i())?,
            0x1b => self.exec_alu_imm32(self.funct3(), self.rd(), self.rs1(), self.imm_i() as i32)?,
            0x33 => self.exec_alu_reg(self.funct3(), self.funct7(), self.rd(), self.rs1(), self.rs2())?,
            0x3b => self.exec_alu_reg32(self.funct3(), self.funct7(), self.rd(), self.rs1(), self.rs2())?,
            0x0f => (),
            0x73 => self.exec_system(self.funct3(), self.rd())?,
            _ => return Err("Unknown opcode"),
        }

        Ok(())
    }

    fn mem_load<T>(&self, addr: u64) -> T {
        let ptr = self.mem_ptr::<T>(addr);
        unsafe { ptr::read(ptr as *const T) }
    }

    fn mem_store<T>(&mut self, addr: u64, value: T) {
        let ptr = self.mem_ptr::<T>(addr) as *mut T;
        unsafe { ptr::write(ptr, value) }
    }

    fn mem_ptr<T>(&self, addr: u64) -> *mut u8 {
        if addr > 0xFFFFFFFFFFFFFFF0u64 {
            panic!("Memory access out of bounds");
        }

        let addr_max = addr.wrapping_add(mem::size_of::<T>() as u64 - 1);

        if addr_max < self.p_end {
            &self.program[addr as usize] as *const _ as *mut u8
        } else if addr >= self.d_beg && addr_max < self.d_end {
            &self.data[(addr - self.d_beg) as usize] as *const _ as *mut u8
        } else if addr >= self.s_beg && addr_max < self.s_end {
            &self.stack[(addr - self.s_beg) as usize] as *const _ as *mut u8
        } else {
            panic!("Memory access out of bounds");
        }
    }

    fn exec_branch(&self, funct3: u8, rs1: u8, rs2: u8, _imm: i64) -> Result<bool, &'static str> {
        let taken = match funct3 {
            0 => self.x[rs1 as usize] == self.x[rs2 as usize],
            1 => self.x[rs1 as usize] != self.x[rs2 as usize],
            4 => (self.x[rs1 as usize] as i64) < (self.x[rs2 as usize] as i64),
            5 => (self.x[rs1 as usize] as i64) >= (self.x[rs2 as usize] as i64),
            6 => self.x[rs1 as usize] < self.x[rs2 as usize],
            7 => self.x[rs1 as usize] >= self.x[rs2 as usize],
            _ => return Err("Unknown branch operation"),
        };

        Ok(taken)
    }

    fn exec_load(&mut self, funct3: u8, rd: u8, rs1: u8, imm: i64) -> Result<(), &'static str> {
        let addr = (self.x[rs1 as usize] as i64 + imm) as u64;
        self.x[rd as usize] = match funct3 {
            0 => self.mem_load::<i8>(addr) as i64 as u64,
            1 => self.mem_load::<i16>(addr) as i64 as u64,
            2 => self.mem_load::<i32>(addr) as i64 as u64,
            3 => self.mem_load::<u64>(addr),
            4 => self.mem_load::<u8>(addr) as u64,
            5 => self.mem_load::<u16>(addr) as u64,
            6 => self.mem_load::<u32>(addr) as u64,
            _ => return Err("Unknown load operation"),
        };

        Ok(())
    }

    fn exec_store(&mut self, funct3: u8, rs1: u8, rs2: u8, imm: i64) -> Result<(), &'static str> {
        let addr = (self.x[rs1 as usize] as i64 + imm) as u64;
        match funct3 {
            0 => self.mem_store::<u8>(addr, self.x[rs2 as usize] as u8),
            1 => self.mem_store::<u16>(addr, self.x[rs2 as usize] as u16),
            2 => self.mem_store::<u32>(addr, self.x[rs2 as usize] as u32),
            3 => self.mem_store::<u64>(addr, self.x[rs2 as usize]),
            _ => return Err("Unknown store operation"),
        };

        Ok(())
    }

    fn exec_alu_imm(&mut self, funct3: u8, rd: u8, rs1: u8, imm: i64) -> Result<(), &'static str> {
        self.x[rd as usize] = match funct3 {
            0 => self.x[rs1 as usize].wrapping_add(imm as u64),
            1 => self.x[rs1 as usize] << (imm & 0x3f) as u64,
            2 => ((self.x[rs1 as usize] as i64) < imm) as u64,
            3 => (self.x[rs1 as usize] < imm as u64) as u64,
            4 => self.x[rs1 as usize] ^ imm as u64,
            5 => if (imm & 0x400) == 0 {
                    self.x[rs1 as usize] >> (imm & 0x3f) as u64
                } else {
                    (self.x[rs1 as usize] as i64 >> (imm & 0x3f)) as u64
                },
            6 => self.x[rs1 as usize] | imm as u64,
            7 => self.x[rs1 as usize] & imm as u64,
            _ => return Err("Unknown alu_imm operation"),
        };

        Ok(())
    }

    fn exec_alu_imm32(&mut self, funct3: u8, rd: u8, rs1: u8, imm: i32) -> Result<(), &'static str> {
        let result: u32 = match funct3 {
            0 => (self.x[rs1 as usize] as u32).wrapping_add(imm as u32),
            1 => (self.x[rs1 as usize] as u32) << (imm & 0x1f),
            5 => if (imm & 0x400) == 0 {
                    self.x[rs1 as usize] as u32 >> (imm & 0x1f)
                } else {
                    (self.x[rs1 as usize] as i32 >> (imm & 0x1f)) as u32
                },
            _ => return Err("Unknown alu_imm32 operation"),
        };

        self.x[rd as usize] = result as i64 as u64; // Sign-extend
        Ok(())
    }

    fn exec_alu_reg(&mut self, funct3: u8, funct7: u8, rd: u8, rs1: u8, rs2: u8) -> Result<(), &'static str> {
        let op = (funct7 << 3) | funct3;
        self.x[rd as usize] = match op {
            0x000 => self.x[rs1 as usize].wrapping_add(self.x[rs2 as usize]),
            0x101 => self.x[rs1 as usize].wrapping_sub(self.x[rs2 as usize]),
            0x001 => self.x[rs1 as usize] << (self.x[rs2 as usize] & 0x3f),
            0x002 => ((self.x[rs1 as usize] as i64) < (self.x[rs2 as usize] as i64)) as u64,
            0x003 => (self.x[rs1 as usize] < self.x[rs2 as usize]) as u64,
            0x004 => self.x[rs1 as usize] ^ self.x[rs2 as usize],
            0x005 => self.x[rs1 as usize] >> (self.x[rs2 as usize] & 0x3f),
            0x105 => (self.x[rs1 as usize] as i64 >> (self.x[rs2 as usize] & 0x3f)) as u64,
            0x006 => self.x[rs1 as usize] | self.x[rs2 as usize],
            0x007 => self.x[rs1 as usize] & self.x[rs2 as usize],

            0x008 => self.x[rs1 as usize].wrapping_mul(self.x[rs2 as usize]),
            0x009 => mulh(self.x[rs1 as usize] as i64, self.x[rs2 as usize] as i64) as u64,
            0x00a => mulhsu(self.x[rs1 as usize] as i64, self.x[rs2 as usize]) as i64 as u64,
            0x00b => mulhu(self.x[rs1 as usize], self.x[rs2 as usize]),

            0x00c => {
                if self.x[rs2 as usize] != 0 {
                    if (self.x[rs1 as usize] as i64 == i64::MIN) && (self.x[rs2 as usize] as i64 == -1) {
                        i64::MIN as u64
                    } else {
                        (self.x[rs1 as usize] as i64 / self.x[rs2 as usize] as i64) as u64
                    }
                } else {
                    u64::MAX
                }
            }

            0x00d => if self.x[rs2 as usize] != 0 {
                self.x[rs1 as usize] / self.x[rs2 as usize]
            } else {
                u64::MAX
            },

            0x00e => {
                if self.x[rs2 as usize] != 0 {
                    if (self.x[rs1 as usize] as i64 == i64::MIN) && (self.x[rs2 as usize] as i64 == -1) {
                        0u64
                    } else {
                        (self.x[rs1 as usize] as i64 % self.x[rs2 as usize] as i64) as u64
                    }
                } else {
                    self.x[rs1 as usize]
                }
            }

            0x00f => if self.x[rs2 as usize] != 0 {
                self.x[rs1 as usize] % self.x[rs2 as usize]
            } else {
                self.x[rs1 as usize]
            },

            _ => return Err("Unknown alu_reg operation"),
        };

        Ok(())
    }

    fn exec_alu_reg32(&mut self, funct3: u8, funct7: u8, rd: u8, rs1: u8, rs2: u8) -> Result<(), &'static str> {
        let op = (funct7 << 3) | funct3;
        let a = self.x[rs1 as usize] as u32;
        let b = self.x[rs2 as usize] as u32;

        let result = match op {
            0x000 => a.wrapping_add(b),
            0x100 => a.wrapping_sub(b),
            0x001 => a << (b & 0x1f),
            0x005 => a >> (b & 0x1f),
            0x105 => (a as i32 >> (b & 0x1f)) as u32,

            0x008 => a.wrapping_mul(b as u64 as u32),
            0x00c => {
                (if b != 0 {
                    if (a as i32) == i32::MIN && (b as i32) == -1 {
                        i32::MIN as i32
                    } else {
                        a as i32 / b as i32
                    }
                } else {
                    i32::MAX
                }) as u32
            }

            0x00d => {
                (if b != 0 {
                    a / b
                } else {
                    u32::MAX
                }) as u32
            }

            0x00e => {
                (if b != 0 {
                    a as i32 % b as i32
                } else {
                    a as i32
                }) as u32
            }

            0x00f => {
                (if b != 0 {
                    a % b
                } else {
                    a
                }) as u32
            }

            _ => return Err("Unknown alu_reg32 operation"),
        };

        self.x[rd as usize] = result as i64 as u64;
        Ok(())
    }

    fn exec_system(&mut self, funct3: u8, _rd: u8) -> Result<(), &'static str> {
        if funct3 != 0 {
            self.handle_csr()?;
            return Ok(());
        }

        match self.inst {
            0x00000073 => {
                self.handle_ecall()?;
            }

            0x00100073 => {
                let has_prev = (self.pc >= 8) && (self.mem_load::<u32>(self.pc - 8) == 0x01f01013u32);
                let has_next = (self.pc + 3 < self.p_end) && (self.mem_load::<u32>(self.pc) == 0x40705013u32);
                if has_prev && has_next {
                    self.handle_semihost()?;
                } else {
                    self.halted.store(true, Ordering::Relaxed);
                }
                return Ok(());
            }

            0x10500073 | 0x30200073 | 0x10200073 | 0x00200073 => {
                return Err("Privilege-mode return instruction not supported in this VM");
            }

            _ => {
                return Err("Unknown SYSTEM instruction");
            }
        }

        Ok(())
    }

    fn handle_csr(&mut self) -> Result<(), &'static str> {
        let d = self.rd();
        if d != 0 {
            self.x[d as usize] = 0;
        }
        Ok(())
    }

    fn handle_semihost(&self) -> Result<(), &'static str> {
        Err("Semihosting call not supported.")
    }

    fn handle_ecall(&self) -> Result<(), &'static str> {
        Err("ECALL not supported")
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
        ((self.inst as i32) >> 20) as i64
    }

    fn imm_s(&self) -> i64 {
        (self.imm_i() & !0x1f) | self.rd() as i64
    }

    fn imm_b(&self) -> i64 {
        (((self.inst & 0x80000000) as i32) >> 19) as i64
            | ((self.inst & 0x80) << 4) as i64
            | ((self.inst >> 20) & 0x7e0) as i64
            | ((self.inst >> 7) & 0x1e) as i64
    }

    fn imm_j(&self) -> i64 {
        (((self.inst & 0x80000000) as i32) >> 11) as i64
            | (self.inst & 0xff000) as i64
            | ((self.inst >> 9) & 0x800) as i64
            | ((self.inst >> 20) & 0x7fe) as i64
    }

    fn imm_u(&self) -> u64 {
        ((self.inst & 0xfffff000) as i32) as u64
    }
}

fn mulh(a: i64, b: i64) -> u64 {
    let ad = a as i128;
    let bd = b as i128;
    ((ad * bd) >> 64) as u64
}

fn mulhu(a: u64, b: u64) -> u64 {
    let ad = a as u128;
    let bd = b as u128;
    ((ad * bd) >> 64) as u64
}

fn mulhsu(a: i64, b: u64) -> u64 {
    let ad = a as i128;
    let bd = b as i128;
    ((ad * bd) >> 64) as u64
}

fn main() {
    // Entry point for the application
}