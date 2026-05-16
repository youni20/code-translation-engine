use std::fs::File;
use std::io::{self, Read};
use std::sync::atomic::{AtomicBool, Ordering};

type U8 = u8;
type U16 = u16;
type U32 = u32;
type U64 = u64;
type I8 = i8;
type I16 = i16;
type I32 = i32;
type I64 = i64;

pub struct VM {
    pc: U64,                          
    inst: U32,                        
    program: Vec<U8>,                 
    x: [U64; 32],                     
    stack: Vec<U8>,                   
    data: Vec<U8>,                    
    halted: AtomicBool,               
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
        let mut vm = VM {
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
        };
        vm.reset();
        vm
    }

    pub fn program_load(&mut self, prog_filename: &str) -> Result<U64, io::Error> {
        self.program = VM::load_program(prog_filename, self.max_prog_size)?;
        self.reset();
        Ok(self.p_beg)
    }

    pub fn program_load_bytes(&mut self, prog: &[U8]) -> Result<U64, &'static str> {
        if prog.len() > self.max_prog_size {
            return Err("Program too large");
        }
        self.program = prog.to_vec();
        self.reset();
        Ok(self.p_beg)
    }

    pub fn map_data_mem(&mut self, mem: &mut [U8]) -> U64 {
        self.data = mem.to_vec();
        self.reset();
        self.d_beg
    }

    pub fn register_set(&mut self, reg: usize, value: U64) -> Result<(), &'static str> {
        if reg >= 32 {
            return Err("Invalid register number");
        }
        if reg != 0 {
            self.x[reg] = value;
        }
        Ok(())
    }

    pub fn register_get(&self, reg: usize) -> Result<U64, &'static str> {
        if reg >= 32 {
            return Err("Invalid register number");
        }
        Ok(self.x[reg])
    }

    pub fn stack_push<T: Copy>(&mut self, val: T) -> U64 {
        self.x[2] -= std::mem::size_of::<T>() as u64;
        self.mem_store(self.x[2], val);
        self.x[2]
    }

    pub fn stack_pop<T: Copy>(&mut self) -> T {
        self.x[2] += std::mem::size_of::<T>() as u64;
        self.mem_load::<T>(self.x[2] - std::mem::size_of::<T>() as u64)
    }

    pub fn stack_peek<T: Copy>(&self) -> T {
        self.mem_load::<T>(self.x[2])
    }

    pub fn execute_program(&mut self, entry_point: U64, max_instructions: usize) -> Result<(), &'static str> {
        let prog_sz = self.program.len();
        let sentinel_pc = ((prog_sz + 3) & !3) as u64;

        self.pc = entry_point;
        self.halted.store(false, Ordering::SeqCst);
        let mut count = 0;

        if prog_sz < 4 {
            return Err("Program too small (must be at least 4 bytes)");
        }

        while !self.halted.load(Ordering::SeqCst) {
            if self.pc > (prog_sz as u64 - 4) {
                return Err("PC jumped program region");
            }
            if count > max_instructions {
                return Err("Maximum instruction count exceeded");
            }

            self.execute_instruction()?;

            if self.pc == sentinel_pc {
                self.halted.store(true, Ordering::SeqCst);
            }

            count += 1;
        }

        Ok(())
    }

    pub fn halt_program(&self) -> bool {
        self.halted.swap(true, Ordering::SeqCst)
    }

    pub fn reset(&mut self) {
        for xn in &mut self.x {
            *xn = 0;
        }
        self.x[1] = (self.program.len() + 3) as U64 & !3;
        self.x[2] = self.program.len() as U64 + 64 + self.data.len() as u64 + 64 + self.stack.len() as u64;
        self.x[8] = self.x[2];

        self.p_end = self.program.len() as U64;
        self.d_beg = self.program.len() as U64 + 64;
        self.d_end = self.program.len() as U64 + 64 + self.data.len() as U64;
        self.s_beg = self.program.len() as U64 + 64 + self.data.len() as u64 + 64;
        self.s_end = self.program.len() as U64 + 64 + self.data.len() as u64 + 64 + self.stack.len() as U64;
    }

    fn load_program(filename: &str, max_size: usize) -> Result<Vec<U8>, io::Error> {
        let mut file = File::open(filename)?;
        let size = file.metadata()?.len();

        if size as usize > max_size {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Program too large"));
        }

        let mut prog = vec![0; size as usize];
        file.read_exact(&mut prog)?;

        Ok(prog)
    }

    fn execute_instruction(&mut self) -> Result<(), &'static str> {
        if self.pc as usize + 4 > self.program.len() {
            return Err("Program counter out of bounds");
        }

        self.inst = U32::from_le_bytes(self.program[self.pc as usize..self.pc as usize + 4].try_into().unwrap());
        self.pc += 4;

        self.x[0] = 0; // Ensure x0 stays zero

        match self.opcode() {
            0x37 => self.x[self.rd() as usize] = self.imm_u(),                                          
            0x17 => self.x[self.rd() as usize] = (self.pc - 4) + self.imm_u(),                          
            0x6f => {
                self.x[self.rd() as usize] = self.pc;
                self.pc = self.pc.wrapping_add(self.imm_j().try_into().unwrap()).wrapping_sub(4);
            } 
            0x67 => {
                let target = (self.x[self.rs1() as usize].wrapping_add(self.imm_i().try_into().unwrap()) & !1) as U64;
                self.x[self.rd() as usize] = self.pc;
                self.pc = target;
            } 
            0x63 => self.exec_branch(self.funct3(), self.rs1(), self.rs2(), self.imm_b())?,           
            0x03 => self.exec_load(self.funct3(), self.rd(), self.rs1(), self.imm_i())?,              
            0x23 => self.exec_store(self.funct3(), self.rs1(), self.rs2(), self.imm_s())?,            
            0x13 => self.exec_alu_imm(self.funct3(), self.rd(), self.rs1(), self.imm_i())?,           
            0x1b => self.exec_alu_imm32(self.funct3(), self.rd(), self.rs1(), self.imm_i() as I32)?,  
            0x33 => self.exec_alu_reg(self.funct3(), self.funct7().into(), self.rd(), self.rs1(), self.rs2())?, 
            0x3b => self.exec_alu_reg32(self.funct3(), self.funct7().into(), self.rd(), self.rs1(), self.rs2())?, 
            0x0f => {} 
            0x73 => self.exec_system(self.funct3(), self.rd())?,                                        
            _ => return Err("Unknown opcode"),
        };

        Ok(())
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
        ((self.inst as I32) >> 20) as I64
    }

    fn imm_s(&self) -> I64 {
        (self.imm_i() & !0x1f) | (self.rd() as I64)
    }

    fn imm_b(&self) -> I64 {
        (((self.inst as I32 & 0x80000000u32 as i32) as I64) >> 19)
            | (((self.inst & 0x80) as I64) << 4)
            | (((self.inst >> 20) & 0x7e0) as I64)
            | (((self.inst >> 7) & 0x1e) as I64)
    }

    fn imm_j(&self) -> I64 {
        (((self.inst as I32 & 0x80000000u32 as i32) as I64) >> 11)
            | (((self.inst & 0xff000) as I64))
            | (((self.inst >> 9) & 0x800) as I64)
            | (((self.inst >> 20) & 0x7fe) as I64)
    }

    fn imm_u(&self) -> U64 {
        (self.inst & 0xfffff000) as U64
    }

    fn mem_ptr<T>(&self, addr: U64) -> Result<*const u8, &'static str> {
        if addr > 0xFFFFFFFFFFFFFFF0u64 {
            return Err("Memory access out of bounds");
        }

        let addr_max = addr + std::mem::size_of::<T>() as u64 - 1;

        if addr_max < self.p_end {
            Ok(self.program.as_ptr().wrapping_add(addr as usize))
        } else if addr >= self.d_beg && addr_max < self.d_end {
            Ok(self.data.as_ptr().wrapping_add((addr - self.d_beg) as usize))
        } else if addr >= self.s_beg && addr_max < self.s_end {
            Ok(self.stack.as_ptr().wrapping_add((addr - self.s_beg) as usize))
        } else {
            Err("Memory access out of bounds")
        }
    }

    fn mem_load<T: Copy>(&self, addr: U64) -> T {
        unsafe {
            let ptr = self.mem_ptr::<T>(addr).unwrap();
            std::ptr::read_unaligned(ptr as *const T)
        }
    }

    fn mem_store<T: Copy>(&mut self, addr: U64, value: T) {
        unsafe {
            let ptr = self.mem_ptr::<T>(addr).unwrap() as *mut T;
            std::ptr::write_unaligned(ptr, value);
        }
    }

    fn exec_branch(&mut self, funct3: U8, rs1: U8, rs2: U8, imm: I64) -> Result<(), &'static str> {
        let taken = match funct3 {
            0 => self.x[rs1 as usize] == self.x[rs2 as usize],                                     
            1 => self.x[rs1 as usize] != self.x[rs2 as usize],                                     
            4 => (self.x[rs1 as usize] as I64) < (self.x[rs2 as usize] as I64),                    
            5 => (self.x[rs1 as usize] as I64) >= (self.x[rs2 as usize] as I64),                   
            6 => self.x[rs1 as usize] < self.x[rs2 as usize],                                      
            7 => self.x[rs1 as usize] >= self.x[rs2 as usize],                                     
            _ => return Err("Unknown branch operation"),
        };
        if taken {
            self.pc = self.pc.wrapping_add(imm as U64).wrapping_sub(4);
        }

        Ok(())
    }

    fn exec_load(&mut self, funct3: U8, rd: U8, rs1: U8, imm: I64) -> Result<(), &'static str> {
        let addr = (self.x[rs1 as usize] as I64).wrapping_add(imm) as U64;
        self.x[rd as usize] = match funct3 {
            0 => self.mem_load::<I8>(addr) as I64 as U64,  
            1 => self.mem_load::<I16>(addr) as I64 as U64, 
            2 => self.mem_load::<I32>(addr) as I64 as U64, 
            3 => self.mem_load::<U64>(addr),               
            4 => self.mem_load::<U8>(addr) as U64,         
            5 => self.mem_load::<U16>(addr) as U64,        
            6 => self.mem_load::<U32>(addr) as U64,        
            _ => return Err("Unknown load operation"),
        };

        Ok(())
    }

    fn exec_store(&mut self, funct3: U8, rs1: U8, rs2: U8, imm: I64) -> Result<(), &'static str> {
        let addr = (self.x[rs1 as usize] as I64).wrapping_add(imm) as U64;
        match funct3 {
            0 => self.mem_store::<U8>(addr, self.x[rs2 as usize] as U8),  
            1 => self.mem_store::<U16>(addr, self.x[rs2 as usize] as U16), 
            2 => self.mem_store::<U32>(addr, self.x[rs2 as usize] as U32), 
            3 => self.mem_store::<U64>(addr, self.x[rs2 as usize]),       
            _ => return Err("Unknown store operation"),
        };

        Ok(())
    }

    fn exec_alu_imm(&mut self, funct3: U8, rd: U8, rs1: U8, imm: I64) -> Result<(), &'static str> {
        self.x[rd as usize] = match funct3 {
            0 => self.x[rs1 as usize].wrapping_add(imm as U64),                                
            1 => self.x[rs1 as usize] << (imm as u64 & 0x3f),                                  
            2 => ((self.x[rs1 as usize] as I64) < imm) as U64,                                 
            3 => (self.x[rs1 as usize] < imm as U64) as U64,                                   
            4 => self.x[rs1 as usize] ^ imm as U64,                                            
            5 => if (imm & 0x400) == 0 {
                self.x[rs1 as usize] >> (imm & 0x3f)
            } else {
                ((self.x[rs1 as usize] as I64) >> (imm & 0x3f)) as U64
            } 
            6 => self.x[rs1 as usize] | imm as U64,                                            
            7 => self.x[rs1 as usize] & imm as U64,                                            
            _ => return Err("Unknown alu_imm operation"),
        };

        Ok(())
    }

    fn exec_alu_imm32(&mut self, funct3: U8, rd: U8, rs1: U8, imm: I32) -> Result<(), &'static str> {
        let result: U32 = match funct3 {
            0 => (self.x[rs1 as usize] as U32).wrapping_add(imm as U32),                       
            1 => (self.x[rs1 as usize] as U32) << (imm & 0x1f),                                
            5 => if (imm & 0x400) == 0 {
                self.x[rs1 as usize] as U32 >> (imm & 0x1f)
            } else {
                ((self.x[rs1 as usize] as I32) >> (imm & 0x1f)) as U32
            } 
            _ => return Err("Unknown alu_imm32 operation"),
        };

        self.x[rd as usize] = result as I32 as I64 as U64; 

        Ok(())
    }

    fn exec_alu_reg(&mut self, funct3: U8, funct7: u16, rd: U8, rs1: U8, rs2: U8) -> Result<(), &'static str> {
        let op = (funct7 << 3) | funct3 as u16;
        self.x[rd as usize] = match op {
            0x00 => self.x[rs1 as usize].wrapping_add(self.x[rs2 as usize]),                  
            0x20 => self.x[rs1 as usize].wrapping_sub(self.x[rs2 as usize]),                  
            0x01 => self.x[rs1 as usize] << (self.x[rs2 as usize] & 0x3f),                    
            0x02 => ((self.x[rs1 as usize] as I64) < self.x[rs2 as usize] as I64) as U64,     
            0x03 => (self.x[rs1 as usize] < self.x[rs2 as usize]) as U64,                     
            0x04 => self.x[rs1 as usize] ^ self.x[rs2 as usize],                             
            0x05 => self.x[rs1 as usize] >> (self.x[rs2 as usize] & 0x3f),                   
            0x15 => ((self.x[rs1 as usize] as I64) >> (self.x[rs2 as usize] & 0x3f)) as U64, 
            0x06 => self.x[rs1 as usize] | self.x[rs2 as usize],                             
            0x07 => self.x[rs1 as usize] & self.x[rs2 as usize],                             

            0x08 => self.x[rs1 as usize].wrapping_mul(self.x[rs2 as usize]),                 
            0x09 => self.mulh(self.x[rs1 as usize] as I64, self.x[rs2 as usize] as I64),     
            0x0a => self.mulhsu(self.x[rs1 as usize] as I64, self.x[rs2 as usize]),          
            0x0b => self.mulhu(self.x[rs1 as usize], self.x[rs2 as usize]),                  
            0x0c => { 
                if self.x[rs2 as usize] != 0 {
                    if self.x[rs1 as usize] == i64::MIN as U64 && self.x[rs2 as usize] as I64 == -1 {
                        i64::MIN as U64
                    } else {
                        (self.x[rs1 as usize] as I64 / self.x[rs2 as usize] as I64) as U64
                    }
                } else {
                    0xFFFFFFFFFFFFFFFF
                }
            }
            0x0d => { 
                if self.x[rs2 as usize] != 0 {
                    self.x[rs1 as usize] / self.x[rs2 as usize]
                } else {
                    0xFFFFFFFFFFFFFFFF
                }
            }
            0x0e => { 
                if self.x[rs2 as usize] != 0 {
                    if self.x[rs1 as usize] == i64::MIN as U64 && self.x[rs2 as usize] as I64 == -1 {
                        0
                    } else {
                        (self.x[rs1 as usize] as I64 % self.x[rs2 as usize] as I64) as U64
                    }
                } else {
                    self.x[rs1 as usize]
                }
            }
            0x0f => { 
                if self.x[rs2 as usize] != 0 {
                    self.x[rs1 as usize] % self.x[rs2 as usize]
                } else {
                    self.x[rs1 as usize]
                }
            }
            _ => return Err("Unknown alu_reg operation"),
        };

        Ok(())
    }

    fn exec_alu_reg32(&mut self, funct3: U8, funct7: u16, rd: U8, rs1: U8, rs2: U8) -> Result<(), &'static str> {
        let op = (funct7 << 3) | funct3 as u16;
        let result: I32 = match op {
            0x00 => (self.x[rs1 as usize] as U32).wrapping_add(self.x[rs2 as usize] as U32).try_into().unwrap(), 
            0x20 => (self.x[rs1 as usize] as U32).wrapping_sub(self.x[rs2 as usize] as U32).try_into().unwrap(), 
            0x01 => ((self.x[rs1 as usize] as U32) << (self.x[rs2 as usize] & 0x1f) as u32).try_into().unwrap(),   
            0x05 => (self.x[rs1 as usize] as U32 >> (self.x[rs2 as usize] & 0x1f) as u32).try_into().unwrap(),   
            0x15 => (self.x[rs1 as usize] as I32 >> (self.x[rs2 as usize] & 0x1f) as i32).try_into().unwrap(),   

            0x08 => (self.x[rs1 as usize] as I32).wrapping_mul(self.x[rs2 as usize] as I32), 
            0x0c => {
                if self.x[rs2 as usize] != 0 {
                    if self.x[rs1 as usize] as I32 == i32::MIN && self.x[rs2 as usize] as I32 == -1 {
                        i32::MIN
                    } else {
                        self.x[rs1 as usize] as I32 / self.x[rs2 as usize] as I32
                    }
                } else {
                    -1
                }
            } 
            0x0d => {
                if self.x[rs2 as usize] != 0 {
                    (self.x[rs1 as usize] as U32 / self.x[rs2 as usize] as U32) as I32
                } else {
                    -1
                }
            } 
            0x0e => {
                if self.x[rs2 as usize] != 0 {
                    self.x[rs1 as usize] as I32 % self.x[rs2 as usize] as I32
                } else {
                    self.x[rs1 as usize] as I32
                }
            } 
            0x0f => {
                if self.x[rs2 as usize] != 0 {
                    (self.x[rs1 as usize] as U32 % self.x[rs2 as usize] as U32) as I32
                } else {
                    self.x[rs1 as usize] as I32
                }
            } 
            _ => return Err("Unknown alu_reg32 operation"),
        };

        self.x[rd as usize] = result as I64 as U64; 

        Ok(())
    }

    fn exec_system(&mut self, funct3: U8, rd: U8) -> Result<(), &'static str> {
        if funct3 != 0 {
            self.handle_csr(rd)?;
            return Ok(());
        }

        match self.inst {
            0x00000073 => { 
                self.handle_ecall()?;
            }
            0x00100073 => { 
                let has_prev = self.pc >= 8 && self.mem_load::<U32>(self.pc - 8) == 0x01f01013;
                let has_next = self.pc + 3 < self.p_end && self.mem_load::<U32>(self.pc) == 0x40705013;

                if has_prev && has_next {
                    self.handle_semihost()?;
                } else {
                    self.halted.store(true, Ordering::SeqCst);
                }
            }
            0x10500073 | 0x10200073 | 0x00200073 => {
                return Err("Privilege-mode return instruction (MRET/SRET/URET) is not supported in this VM; this VM has no privilege levels");
            }
            0x30200073 => {
                return Err("WFI (wait-for-interrupt) is not supported in this VM; remove interrupt-driven idle loops from bare-metal code");
            }
            _ => {
                return Err("Unknown SYSTEM instruction");
            }
        }

        Ok(())
    }

    fn handle_csr(&mut self, rd: U8) -> Result<(), &'static str> {
        if rd != 0 {
            self.x[rd as usize] = 0;
        }
        Ok(())
    }

    fn handle_semihost(&mut self) -> Result<(), &'static str> {
        Err("Semihosting call is not supported in this VM; implement handle_semihost() to support semihosting operations")
    }

    fn handle_ecall(&mut self) -> Result<(), &'static str> {
        Err("ECALL is not supported in this VM; implement handle_ecall() to support system calls")
    }

    fn mulh(&self, a: I64, b: I64) -> U64 {
        let neg = (a < 0) ^ (b < 0);
        let abs_a = a.abs() as U64;
        let abs_b = b.abs() as U64;
        let (hi, lo) = self.mulu64_128(abs_a, abs_b);

        if neg {
            !hi.wrapping_sub(if lo == 0 { 1 } else { 0 })
        } else {
            hi
        }
    }

    fn mulhsu(&self, a: I64, b: U64) -> U64 {
        let abs_a = a.abs() as U64;
        let (hi, lo) = self.mulu64_128(abs_a, b);

        if a < 0 {
            !hi.wrapping_sub(if lo == 0 { 1 } else { 0 })
        } else {
            hi
        }
    }

    fn mulhu(&self, a: U64, b: U64) -> U64 {
        self.mulu64_128(a, b).0
    }

    fn mulu64_128(&self, a: U64, b: U64) -> (U64, U64) {
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
}

fn main() {
}