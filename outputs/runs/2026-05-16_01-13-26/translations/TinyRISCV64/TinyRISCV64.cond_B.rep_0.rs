use std::fs::File;
use std::io::{Read, BufReader};
use std::sync::atomic::{AtomicBool, Ordering};
use std::convert::TryInto;
use std::error::Error;

struct VM {
    pc: u64,                        // Program counter
    inst: u32,                      // Current instruction
    program: Vec<u8>,               // Program memory
    x: [u64; 32],                   // Registers x0-x31
    stack: Vec<u8>,                 // Stack memory
    data: &'static mut [u8],        // Data memory
    halted: AtomicBool,             // Program exited or externally halted
    max_prog_size: usize,           // Maximum allowed program image size (bytes)

    // Virtual addressing:
    p_end: u64,      // Program mem end
    d_beg: u64,      // Data mem begin
    d_end: u64,      // Data mem end
    s_beg: u64,      // Stack mem begin
    s_end: u64,      // Stack mem end
}

const P_BEG: u64 = 0;

impl VM {
    const P_BEG: u64 = 0;
    pub fn new(stack_size: usize, max_program_size: usize) -> VM {
        VM {
            pc: 0,
            inst: 0,
            program: Vec::new(),
            x: [0; 32],
            stack: vec![0; stack_size],
            data: &mut [],
            halted: AtomicBool::new(false),
            max_prog_size: max_program_size,
            p_end: 0,
            d_beg: 0,
            d_end: 0,
            s_beg: 0,
            s_end: 0,
        }
    }

    pub fn program_load(&mut self, prog_filename: &str) -> Result<u64, Box<dyn Error>> {
        self.program = Self::load_program(prog_filename, self.max_prog_size)?;
        self.reset();
        Ok(P_BEG)
    }

    pub fn program_load_from_data(&mut self, prog: &[u8]) -> Result<u64, Box<dyn Error>> {
        if prog.len() > self.max_prog_size {
            return Err(format!("Program too large (max {} bytes)", self.max_prog_size).into());
        }
        self.program = prog.to_vec();
        self.reset();
        Ok(P_BEG)
    }

    pub fn map_data_mem(&mut self, mem: &'static mut [u8]) -> u64 {
        self.data = mem;
        self.reset();
        self.d_beg
    }

    pub fn register_set(&mut self, reg: usize, value: u64) -> Result<(), Box<dyn Error>> {
        if reg >= 32 {
            return Err("Invalid register number".into());
        }
        if reg != 0 {
            self.x[reg] = value;
        }
        Ok(())
    }

    pub fn register_get(&self, reg: usize) -> Result<u64, Box<dyn Error>> {
        if reg >= 32 {
            return Err("Invalid register number".into());
        }
        Ok(self.x[reg])
    }

    pub fn stack_push<T>(&mut self, val: T) -> u64 where T: Copy {
        let size = std::mem::size_of::<T>();
        self.x[2] = self.x[2].overflowing_sub(size as u64).0;
        self.mem_store(self.x[2], val);
        self.x[2]
    }

    pub fn stack_pop<T>(&mut self) -> T where T: Copy + Default + PartialEq {
        let size = std::mem::size_of::<T>();
        self.x[2] += size as u64;
        self.mem_load::<T>(self.x[2] - size as u64)
    }

    pub fn stack_peek<T>(&self) -> T where T: Copy + Default + PartialEq {
        self.mem_load::<T>(self.x[2])
    }

    pub fn execute_program(&mut self, entry_point: u64, max_instructions: usize) -> Result<(), Box<dyn Error>> {
        let prog_sz = self.program.len();
        let sentinel_pc = ((prog_sz + 3) & !3) as u64;
        self.pc = entry_point;
        self.halted.store(false, Ordering::SeqCst);
        let mut count = 0;

        if prog_sz < 4 {
            return Err("Program too small (must be at least 4 bytes)".into());
        }

        while !self.halted.load(Ordering::SeqCst) {
            if self.pc > (prog_sz - 4) as u64 {
                return Err("PC jumped program region".into());
            }
            if count > max_instructions {
                return Err("Maximum instruction count exceeded".into());
            }
            self.execute_instruction()?;
            if self.pc == sentinel_pc {
                self.halted.store(true, Ordering::SeqCst);
            }
            count += 1;
        }
        Ok(())
    }

    pub fn halt_program(&mut self) -> bool {
        !self.halted.swap(true, Ordering::SeqCst)
    }

    pub fn reset(&mut self) {
        for xn in &mut self.x {
            *xn = 0;
        }
        self.x[1] = (self.program.len() + 3) as u64 & !3;
        self.x[2] = self.program.len() as u64 + 64 + self.data.len() as u64 + 64 + self.stack.len() as u64;
        self.x[8] = self.x[2];

        self.p_end = self.program.len() as u64;
        self.d_beg = self.program.len() as u64 + 64;
        self.d_end = self.program.len() as u64 + 64 + self.data.len() as u64;
        self.s_beg = self.program.len() as u64 + 64 + self.data.len() as u64 + 64;
        self.s_end = self.program.len() as u64 + 64 + self.data.len() as u64 + 64 + self.stack.len() as u64;
    }

    fn load_program(filename: &str, max_size: usize) -> Result<Vec<u8>, Box<dyn Error>> {
        let file = File::open(filename)?;
        let mut reader = BufReader::new(file);
        let mut prog = Vec::new();

        reader.read_to_end(&mut prog)?;

        if prog.len() > max_size {
            return Err(format!("Program too large (max {})", max_size).into());
        }
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
        (self.imm_i() & !0x1f_i64) | self.rd() as i64
    }

    fn imm_b(&self) -> i64 {
        (((self.inst & 0x80000000) as i32 as i64) >> 19) |
        (((self.inst & 0x80) << 4) as i64) |
        (((self.inst >> 20) & 0x7e0) as i64) |
        (((self.inst >> 7) & 0x1e) as i64)
    }

    fn imm_j(&self) -> i64 {
        (((self.inst & 0x80000000) as i32 as i64) >> 11) |
        ((self.inst & 0xff000) as i64) |
        (((self.inst >> 9) & 0x800) as i64) |
        (((self.inst >> 20) & 0x7fe) as i64)
    }

    fn imm_u(&self) -> u64 {
        ((self.inst & 0xfffff000) as i32 as i64) as u64
    }

    fn execute_instruction(&mut self) -> Result<(), Box<dyn Error>> {
        self.inst = u32::from_le_bytes(self.program[self.pc as usize..(self.pc as usize) + 4].try_into()?);
        self.pc = self.pc.overflowing_add(4).0;

        self.x[0] = 0;

        match self.opcode() {
            0x37 => self.x[self.rd() as usize] = self.imm_u(),
            0x17 => self.x[self.rd() as usize] = (self.pc - 4).overflowing_add(self.imm_u()).0,
            0x6f => {
                self.x[self.rd() as usize] = self.pc;
                self.pc = self.pc.overflowing_add(self.imm_j() as u64).0.overflowing_sub(4).0;
            }
            0x67 => {
                let target = (self.x[self.rs1() as usize].overflowing_add(self.imm_i() as u64).0) & !1_u64;
                self.x[self.rd() as usize] = self.pc;
                self.pc = target;
            }
            0x63 => self.exec_branch(self.funct3(), self.rs1(), self.rs2(), self.imm_b()),
            0x03 => self.exec_load(self.funct3(), self.rd(), self.rs1(), self.imm_i()),
            0x23 => self.exec_store(self.funct3(), self.rs1(), self.rs2(), self.imm_s()),
            0x13 => self.exec_alu_imm(self.funct3(), self.rd(), self.rs1(), self.imm_i()),
            0x1b => self.exec_alu_imm32(self.funct3(), self.rd(), self.rs1(), self.imm_i() as i32),
            0x33 => self.exec_alu_reg(self.funct3(), self.funct7() as u16, self.rd(), self.rs1(), self.rs2()),
            0x3b => self.exec_alu_reg32(self.funct3(), self.funct7() as u16, self.rd(), self.rs1(), self.rs2()),
            0x0f => {},
            0x73 => self.exec_system(self.funct3(), self.rd()),
            _ => return Err("Unknown opcode".into()),
        }
        Ok(())
    }

    fn mem_ptr<T>(&self, addr: u64) -> Result<&[u8], Box<dyn Error>> {
        if addr > 0xFFFFFFFFFFFFFFF0u64 {
            return Err("Memory access out of bounds".into());
        }

        let addr_max = addr.overflowing_add(std::mem::size_of::<T>() as u64 - 1).0;

        if addr_max < self.p_end {
            Ok(&self.program[addr as usize..])
        } else if addr >= self.d_beg && addr_max < self.d_end {
            Ok(&self.data[(addr - self.d_beg) as usize..])
        } else if addr >= self.s_beg && addr_max < self.s_end {
            Ok(&self.stack[(addr - self.s_beg) as usize..])
        } else {
            Err("Memory access out of bounds".into())
        }
    }

    fn mem_load<T>(&self, addr: u64) -> T
    where T: Default + PartialEq + Copy {
        let size = std::mem::size_of::<T>();
        if let Ok(mem_slice) = self.mem_ptr::<T>(addr) {
            let mut array = vec![0u8; size];
            let slice_len = size.min(mem_slice.len());
            array[..slice_len].copy_from_slice(&mem_slice[..slice_len]);
            unsafe { *(array.as_ptr() as *const T) }
        } else {
            T::default()
        }
    }

    fn mem_store<T>(&mut self, addr: u64, value: T) {
        let size = std::mem::size_of::<T>();
        let slice_range = addr as usize..(addr as usize + size);

        if let Ok(mem_slice) = self.mem_ptr::<T>(addr) {
            let len = size.min(mem_slice.len());
            let ptr = if addr < self.p_end {
                &mut self.program[slice_range.start..slice_range.start + len]
            } else if addr >= self.d_beg && addr < self.d_end {
                &mut self.data[(addr - self.d_beg) as usize..(addr - self.d_beg) as usize + len]
            } else {
                &mut self.stack[(addr - self.s_beg) as usize..(addr - self.s_beg) as usize + len]
            };
            unsafe { *(ptr.as_mut_ptr() as *mut T) = value }
        } 
    }

    fn exec_branch(&mut self, funct3: u8, rs1: u8, rs2: u8, imm: i64) {
        let taken = match funct3 {
            0 => self.x[rs1 as usize] == self.x[rs2 as usize],
            1 => self.x[rs1 as usize] != self.x[rs2 as usize],
            4 => (self.x[rs1 as usize] as i64) < self.x[rs2 as usize] as i64,
            5 => (self.x[rs1 as usize] as i64) >= self.x[rs2 as usize] as i64,
            6 => self.x[rs1 as usize] < self.x[rs2 as usize],
            7 => self.x[rs1 as usize] >= self.x[rs2 as usize],
            _ => false,
        };
        if taken {
            self.pc = self.pc.overflowing_add(imm as u64).0.overflowing_sub(4).0;
        }
    }

    fn exec_load(&mut self, funct3: u8, rd: u8, rs1: u8, imm: i64) {
        let addr = self.x[rs1 as usize].overflowing_add(imm as u64).0;
        self.x[rd as usize] = match funct3 {
            0 => self.mem_load::<i8>(addr) as i64 as u64,
            1 => self.mem_load::<i16>(addr) as i64 as u64,
            2 => self.mem_load::<i32>(addr) as i64 as u64,
            3 => self.mem_load::<u64>(addr),
            4 => self.mem_load::<u8>(addr) as u64,
            5 => self.mem_load::<u16>(addr) as u64,
            6 => self.mem_load::<u32>(addr) as u64,
            _ => 0,
        }
    }

    fn exec_store(&mut self, funct3: u8, rs1: u8, rs2: u8, imm: i64) {
        let addr = self.x[rs1 as usize].overflowing_add(imm as u64).0;
        match funct3 {
            0 => self.mem_store::<u8>(addr, self.x[rs2 as usize] as u8),
            1 => self.mem_store::<u16>(addr, self.x[rs2 as usize] as u16),
            2 => self.mem_store::<u32>(addr, self.x[rs2 as usize] as u32),
            3 => self.mem_store::<u64>(addr, self.x[rs2 as usize]),
            _ => {}
        }
    }

    fn exec_alu_imm(&mut self, funct3: u8, rd: u8, rs1: u8, imm: i64) {
        self.x[rd as usize] = match funct3 {
            0 => self.x[rs1 as usize].overflowing_add(imm as u64).0,
            1 => self.x[rs1 as usize] << (self.imm_i() & 0x3f),
            2 => ((self.x[rs1 as usize] as i64) < imm) as u64,
            3 => (self.x[rs1 as usize] < imm as u64) as u64,
            4 => self.x[rs1 as usize] ^ imm as u64,
            5 => if self.imm_i() & 0x400 == 0 {
                self.x[rs1 as usize] >> (self.imm_i() & 0x3f)
            } else {
                (self.x[rs1 as usize] as i64 >> (self.imm_i() & 0x3f)) as u64
            },
            6 => self.x[rs1 as usize] | imm as u64,
            7 => self.x[rs1 as usize] & imm as u64,
            _ => 0,
        };
    }

    fn exec_alu_imm32(&mut self, funct3: u8, rd: u8, rs1: u8, imm: i32) {
        let result = match funct3 {
            0 => (self.x[rs1 as usize] as u32).wrapping_add(imm as u32),
            1 => (self.x[rs1 as usize] as u32).wrapping_shl(imm as u32 & 0x1f),
            5 => if imm & 0x400 == 0 {
                (self.x[rs1 as usize] as u32).wrapping_shr(imm as u32 & 0x1f)
            } else {
                (self.x[rs1 as usize] as i32 >> (imm & 0x1f) as i32) as u32
            },
            _ => 0,
        };
        self.x[rd as usize] = result as i32 as i64 as u64;
    }

    fn exec_alu_reg(&mut self, funct3: u8, funct7: u16, rd: u8, rs1: u8, rs2: u8) {
        let op = funct7 << 3 | funct3 as u16;
        self.x[rd as usize] = match op {
            0x000 => self.x[rs1 as usize].overflowing_add(self.x[rs2 as usize]).0,
            0x020 => self.x[rs1 as usize].overflowing_sub(self.x[rs2 as usize]).0,
            0x001 => self.x[rs1 as usize] << (self.x[rs2 as usize] & 0x3f),
            0x002 => ((self.x[rs1 as usize] as i64) < self.x[rs2 as usize] as i64) as u64,
            0x003 => (self.x[rs1 as usize] < self.x[rs2 as usize]) as u64,
            0x004 => self.x[rs1 as usize] ^ self.x[rs2 as usize],
            0x005 => self.x[rs1 as usize] >> (self.x[rs2 as usize] & 0x3f),
            0x106 => ((self.x[rs1 as usize] as i64) >> (self.x[rs2 as usize] & 0x3f)) as u64,
            0x006 => self.x[rs1 as usize] | self.x[rs2 as usize],
            0x007 => self.x[rs1 as usize] & self.x[rs2 as usize],
            0x008 => self.x[rs1 as usize].overflowing_mul(self.x[rs2 as usize]).0,
            0x009 => Self::mulh(self.x[rs1 as usize] as i64, self.x[rs2 as usize] as i64) as u64,
            0x00a => Self::mulhsu(self.x[rs1 as usize] as i64, self.x[rs2 as usize]),
            0x00b => Self::mulhu(self.x[rs1 as usize], self.x[rs2 as usize]),
            0x00c => if self.x[rs2 as usize] != 0 {
                if (self.x[rs1 as usize] as i64 == i64::MIN) && (self.x[rs2 as usize] as i64 == -1) {
                    i64::MIN as u64
                } else {
                    ((self.x[rs1 as usize] as i64) / (self.x[rs2 as usize] as i64)) as u64
                }
            } else {
                u64::MAX
            },
            0x00d => if self.x[rs2 as usize] != 0 {
                self.x[rs1 as usize] / self.x[rs2 as usize]
            } else {
                u64::MAX
            },
            0x00e => if self.x[rs2 as usize] != 0 {
                if (self.x[rs1 as usize] as i64 == i64::MIN) && (self.x[rs2 as usize] as i64 == -1) {
                    0u64
                } else {
                    ((self.x[rs1 as usize] as i64) % (self.x[rs2 as usize] as i64)) as u64
                }
            } else {
                self.x[rs1 as usize]
            },
            0x00f => if self.x[rs2 as usize] != 0 {
                self.x[rs1 as usize] % self.x[rs2 as usize]
            } else {
                self.x[rs1 as usize]
            },
            _ => 0,
        };
    }

    fn exec_alu_reg32(&mut self, funct3: u8, funct7: u16, rd: u8, rs1: u8, rs2: u8) {
        let op = funct7 << 3 | funct3 as u16;
        let a = self.x[rs1 as usize] as u32;
        let b = self.x[rs2 as usize] as u32;

        self.x[rd as usize] = match op {
            0x000 => (a.wrapping_add(b) as i32) as i64 as u64,
            0x020 => (a.wrapping_sub(b) as i32) as i64 as u64,
            0x001 => (a << (b & 0x1f)) as i32 as i64 as u64,
            0x005 => (a >> (b & 0x1f)) as i32 as i64 as u64,
            0x106 => (a as i32 >> (b & 0x1f) as i32) as i64 as u64,
            0x008 => (a.wrapping_mul(b) as i32) as i64 as u64,
            0x00c => if b != 0 {
                if (a as i32 == i32::MIN) && (b as i32 == -1) {
                    i32::MIN as i64 as u64
                } else {
                    ((a as i32) / (b as i32)) as i64 as u64
                }
            } else {
                u32::MAX as i64 as u64
            },
            0x00d => if b != 0 {
                (a / b) as i64 as u64
            } else {
                u32::MAX as i64 as u64
            },
            0x00e => if b != 0 {
                (a as i32 % b as i32) as i64 as u64
            } else {
                a as i64 as u64
            },
            0x00f => if b != 0 {
                (a % b) as i64 as u64
            } else {
                a as i64 as u64
            },
            _ => 0,
        };
    }

    fn exec_system(&mut self, funct3: u8, rd: u8) {
        if funct3 != 0 {
            self.handle_csr(rd);
            return;
        }

        match self.inst {
            0x00000073 => self.handle_ecall(),
            0x00100073 => {
                if self.pc >= 8 && self.mem_load::<u32>(self.pc - 8) == 0x01f01013 && self.pc + 3 < self.p_end && self.mem_load::<u32>(self.pc) == 0x40705013 {
                    self.handle_semihost();
                } else {
                    self.halted.store(true, Ordering::SeqCst);
                }
            }
            0x10500073 | 0x30200073 | 0x10200073 | 0x00200073 => {
                self.halted.store(true, Ordering::SeqCst);
            }
            _ => {}
        }
    }

    fn handle_csr(&mut self, rd: u8) {
        if rd != 0 {
            self.x[rd as usize] = 0;
        }
    }

    fn handle_semihost(&self) {
        panic!("Semihosting call at PC: {:#x} is not supported", self.pc - 4);
    }

    fn handle_ecall(&self) {
        panic!("ECALL at PC: {:#x} is not supported", self.pc - 4);
    }

    fn mulh(a: i64, b: i64) -> i64 {
        let result = ((a as i128) * (b as i128)) >> 64;
        result as i64
    }

    fn mulhu(a: u64, b: u64) -> u64 {
        let result = ((a as u128) * (b as u128)) >> 64;
        result as u64
    }

    fn mulhsu(a: i64, b: u64) -> u64 {
        let result = ((a as i128) * (b as i128)) >> 64;
        result as i64 as u64
    }
}

fn main() {
    // The main function is intentionally left empty
    // to satisfy the compiler requirement.
}