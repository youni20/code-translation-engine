/*
 * TinyRISCV64 - RV64IM Virtual Machine
 *
 * https://github.com/neilstephens/TinyRISCV64
 *
 * This is a derivative work based on tinyriscv by inixyz (https://github.com/inixyz/tinyriscv)
 * The core instruction processing logic was ported from C to C++,
 * converted from handling RV32IM to RV64IM, and
 * transplanted into this new class.
 *
 * MIT License
 *
 * Original work Copyright (c) 2023 Alexandru-Florin Ene
 * Modified work Copyright (c) 2025 Neil Stephens
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

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::convert::TryInto;

type U8 = u8;
type U16 = u16;
type U32 = u32;
type U64 = u64;
type I8 = i8;
type I16 = i16;
type I32 = i32;
type I64 = i64;

#[allow(dead_code)]
struct VM {
    pc: U64,
    inst: U32,
    program: Vec<U8>,
    x: [U64; 32],
    stack: Vec<U8>,
    data: Vec<U8>,
    halted: Arc<AtomicBool>,
    max_prog_size: usize,
    p_end: U64,
    d_beg: U64,
    d_end: U64,
    s_beg: U64,
    s_end: U64,
}

impl VM {
    const P_BEG: U64 = 0;

    fn new(stack_size: usize, max_program_size: usize) -> Self {
        let mut vm = VM {
            pc: 0,
            inst: 0,
            program: Vec::new(),
            x: [0; 32],
            stack: vec![0; stack_size],
            data: Vec::new(),
            halted: Arc::new(AtomicBool::new(false)),
            max_prog_size: max_program_size,
            p_end: 0,
            d_beg: 0,
            d_end: 0,
            s_beg: 0,
            s_end: 0,
        };
        vm.reset();
        vm
    }

    pub fn program_load(&mut self, prog_filename: &str) -> Result<U64, String> {
        self.program = VM::load_program(prog_filename, self.max_prog_size)?;
        self.reset();
        Ok(Self::P_BEG)
    }

    pub fn program_load_from_bytes(&mut self, prog: &[U8]) -> Result<U64, String> {
        if prog.len() > self.max_prog_size {
            return Err(format!("Program too large (max {} bytes)", self.max_prog_size));
        }
        self.program = prog.to_vec();
        self.reset();
        Ok(Self::P_BEG)
    }

    pub fn map_data_mem(&mut self, mem: Vec<U8>) -> Result<U64, String> {
        self.data = mem;
        self.reset();
        Ok(self.d_beg)
    }

    pub fn register_set(&mut self, reg: usize, value: U64) -> Result<(), String> {
        if reg >= 32 {
            return Err(String::from("Invalid register number"));
        }
        if reg != 0 {
            self.x[reg] = value;
        }
        Ok(())
    }

    pub fn register_get(&self, reg: usize) -> Result<U64, String> {
        if reg >= 32 {
            return Err(String::from("Invalid register number"));
        }
        Ok(self.x[reg])
    }

    pub fn stack_push<T: Copy>(&mut self, val: T) -> U64 {
        let size = std::mem::size_of::<T>() as U64;
        self.x[2] -= size;
        self.mem_store(self.x[2], val);
        self.x[2]
    }

    pub fn stack_pop<T: Copy>(&mut self) -> T {
        let size = std::mem::size_of::<T>() as U64;
        self.x[2] += size;
        self.mem_load::<T>(self.x[2] - size)
    }

    pub fn stack_peek<T: Copy>(&self) -> T {
        self.mem_load::<T>(self.x[2])
    }

    pub fn execute_program(&mut self, entry_point: U64, max_instructions: usize) -> Result<(), String> {
        let prog_sz = self.program.len() as U64;
        let sentinel_pc = (prog_sz + 3) & !3;

        self.pc = entry_point;
        self.halted.store(false, Ordering::SeqCst);
        let mut count = 0;

        if prog_sz < 4 {
            return Err(String::from("Program too small (must be at least 4 bytes)"));
        }

        while !self.halted.load(Ordering::SeqCst) {
            if self.pc > prog_sz - 4 {
                return Err(String::from("PC jumped program region"));
            }
            if count > max_instructions {
                return Err(String::from("Maximum instruction count exceeded"));
            }
            count += 1;

            self.execute_instruction()?;

            if self.pc == sentinel_pc {
                self.halted.store(true, Ordering::SeqCst);
            }
        }
        Ok(())
    }

    pub fn halt_program(&self) -> bool {
        let already_halted = self.halted.swap(true, Ordering::SeqCst);
        !already_halted
    }

    pub fn reset(&mut self) {
        for xn in self.x.iter_mut() {
            *xn = 0;
        }
        self.x[1] = (self.program.len() as U64 + 3) & !3;
        self.x[2] = self.program.len() as U64 + 64 + self.data.len() as U64 + 64 + self.stack.len() as U64;
        self.x[8] = self.x[2];

        self.p_end = self.program.len() as U64;
        self.d_beg = self.program.len() as U64 + 64;
        self.d_end = self.program.len() as U64 + 64 + self.data.len() as U64;
        self.s_beg = self.program.len() as U64 + 64 + self.data.len() as U64 + 64;
        self.s_end = self.program.len() as U64 + 64 + self.data.len() as U64 + 64 + self.stack.len() as U64;
    }

    fn load_program(filename: &str, max_size: usize) -> Result<Vec<U8>, String> {
        let mut fin = File::open(filename).map_err(|_| format!("Failed to open program file: {}", filename))?;
        let size = fin.seek(SeekFrom::End(0)).map_err(|_| format!("Failed to determine size of file: {}", filename))?;
        if size > max_size as u64 {
            return Err(format!("Program too large (max {})", max_size));
        }
        fin.seek(SeekFrom::Start(0)).map_err(|_| format!("Failed to reset file position: {}", filename))?;
        let mut prog = vec![0; size as usize];
        fin.read_exact(&mut prog).map_err(|_| format!("Failed to read file: {}", filename))?;
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
        (
            ((self.inst as I32 & 0x80000000u32 as i32) >> 19) 
            | ((self.inst as I32 & 0x80) << 4) 
            | ((self.inst >> 20) & 0x7e0) as I32 
            | ((self.inst >> 7) & 0x1e) as I32
        ) as I64
    }
    
    fn imm_j(&self) -> I64 {
        (
            ((self.inst as I32 & 0x80000000u32 as i32) >> 11) 
            | (((self.inst & 0xff000) >> 12) as I32) 
            | (((self.inst >> 9) & 0x800) as I32) 
            | (((self.inst >> 20) & 0x7fe) as I32)
        ) as I64
    }
    
    fn imm_u(&self) -> U64 {
        (self.inst & 0xfffff000) as I64 as U64
    }
    
    fn execute_instruction(&mut self) -> Result<(), String> {
        let inst_bytes = &self.program[self.pc as usize..(self.pc as usize + 4)];
        self.inst = u32::from_le_bytes(inst_bytes.try_into().unwrap());
        self.pc += 4;

        self.x[0] = 0;

        match self.opcode() {
            0x37 => self.x[self.rd() as usize] = self.imm_u(),
            0x17 => self.x[self.rd() as usize] = self.pc - 4 + self.imm_u(),
            0x6f => {
                self.x[self.rd() as usize] = self.pc;
                self.pc = self.pc.wrapping_add(self.imm_j() as U64).wrapping_sub(4);
            }
            0x67 => {
                let target = (self.x[self.rs1() as usize] + self.imm_i() as U64) & !1;
                self.x[self.rd() as usize] = self.pc;
                self.pc = target;
            }
            0x63 => self.exec_branch(self.funct3(), self.rs1(), self.rs2(), self.imm_b() as U64)?,
            0x03 => self.exec_load(self.funct3(), self.rd(), self.rs1(), self.imm_i() as U64)?,
            0x23 => self.exec_store(self.funct3(), self.rs1(), self.rs2(), self.imm_s() as U64)?,
            0x13 => self.exec_alu_imm(self.funct3(), self.rd(), self.rs1(), self.imm_i() as U64)?,
            0x1b => self.exec_alu_imm32(self.funct3(), self.rd(), self.rs1(), self.imm_i() as I32)?,
            0x33 => self.exec_alu_reg(self.funct3(), self.funct7(), self.rd(), self.rs1(), self.rs2())?,
            0x3b => self.exec_alu_reg32(self.funct3(), self.funct7(), self.rd(), self.rs1(), self.rs2())?,
            0x0f => {}
            0x73 => self.exec_system(self.funct3(), self.rd())?,
            _ => return Err(String::from("Unknown opcode")),
        }
        Ok(())
    }

    fn mem_ptr<T>(&self, addr: U64) -> Result<&[u8], String> {
        if addr > 0xFFFFFFFFFFFFFFF0 {
            return Err(String::from("Memory access out of bounds"));
        }

        let addr_max = addr + std::mem::size_of::<T>() as U64 - 1;

        if addr_max < self.p_end {
            Ok(&self.program[addr as usize..])
        } else if addr >= self.d_beg && addr_max < self.d_end {
            Ok(&self.data[addr.saturating_sub(self.d_beg) as usize..])
        } else if addr >= self.s_beg && addr_max < self.s_end {
            Ok(&self.stack[addr.saturating_sub(self.s_beg) as usize..])
        } else {
            Err(String::from("Memory access out of bounds"))
        }
    }

    fn mem_load<T: Copy>(&self, addr: U64) -> T {
        let ptr = self.mem_ptr::<T>(addr).expect("Memory load error");
        unsafe { *(ptr.as_ptr() as *const T) }
    }

    fn mem_store<T: Copy>(&mut self, addr: U64, value: T) {
        let ptr = self.mem_ptr::<T>(addr).expect("Memory store error");
        unsafe {
            (ptr.as_ptr() as *mut T).write(value);
        }
    }

    fn exec_branch(&mut self, funct3: U8, rs1: U8, rs2: U8, imm: U64) -> Result<(), String> {
        let taken = match funct3 {
            0 => self.x[rs1 as usize] == self.x[rs2 as usize],
            1 => self.x[rs1 as usize] != self.x[rs2 as usize],
            4 => (self.x[rs1 as usize] as I64) < (self.x[rs2 as usize] as I64),
            5 => (self.x[rs1 as usize] as I64) >= (self.x[rs2 as usize] as I64),
            6 => self.x[rs1 as usize] < self.x[rs2 as usize],
            7 => self.x[rs1 as usize] >= self.x[rs2 as usize],
            _ => return Err(String::from("Unknown branch operation")),
        };
        if taken {
            self.pc = self.pc.wrapping_add(imm).wrapping_sub(4);
        }
        Ok(())
    }

    fn exec_load(&mut self, funct3: U8, rd: U8, rs1: U8, imm: U64) -> Result<(), String> {
        let addr = self.x[rs1 as usize].wrapping_add(imm);
        match funct3 {
            0 => self.x[rd as usize] = self.mem_load::<I8>(addr) as I64 as U64,
            1 => self.x[rd as usize] = self.mem_load::<I16>(addr) as I64 as U64,
            2 => self.x[rd as usize] = self.mem_load::<I32>(addr) as I64 as U64,
            3 => self.x[rd as usize] = self.mem_load::<U64>(addr),
            4 => self.x[rd as usize] = self.mem_load::<U8>(addr) as U64,
            5 => self.x[rd as usize] = self.mem_load::<U16>(addr) as U64,
            6 => self.x[rd as usize] = self.mem_load::<U32>(addr) as U64,
            _ => return Err(String::from("Unknown load operation")),
        }
        Ok(())
    }

    fn exec_store(&mut self, funct3: U8, rs1: U8, rs2: U8, imm: U64) -> Result<(), String> {
        let addr = self.x[rs1 as usize].wrapping_add(imm);
        match funct3 {
            0 => self.mem_store::<U8>(addr, self.x[rs2 as usize] as U8),
            1 => self.mem_store::<U16>(addr, self.x[rs2 as usize] as U16),
            2 => self.mem_store::<U32>(addr, self.x[rs2 as usize] as U32),
            3 => self.mem_store::<U64>(addr, self.x[rs2 as usize]),
            _ => return Err(String::from("Unknown store operation")),
        }
        Ok(())
    }

    fn exec_alu_imm(&mut self, funct3: U8, rd: U8, rs1: U8, imm: U64) -> Result<(), String> {
        let rs1_idx = rs1 as usize;
        let rd_idx = rd as usize;
        match funct3 {
            0 => self.x[rd_idx] = self.x[rs1_idx].wrapping_add(imm),
            1 => self.x[rd_idx] = self.x[rs1_idx] << (imm as U64 & 0x3f),
            2 => self.x[rd_idx] = if (self.x[rs1_idx] as I64) < imm as I64 { 1 } else { 0 },
            3 => self.x[rd_idx] = if self.x[rs1_idx] < imm { 1 } else { 0 },
            4 => self.x[rd_idx] = self.x[rs1_idx] ^ imm,
            5 => {
                if imm & 0x400 == 0 {
                    self.x[rd_idx] = self.x[rs1_idx] >> (imm as U64 & 0x3f)
                } else {
                    self.x[rd_idx] = (self.x[rs1_idx] as I64 >> (imm & 0x3f)) as U64
                }
            }
            6 => self.x[rd_idx] = self.x[rs1_idx] | imm,
            7 => self.x[rd_idx] = self.x[rs1_idx] & imm,
            _ => return Err(String::from("Unknown ALU immediate operation")),
        }
        Ok(())
    }

    fn exec_alu_imm32(&mut self, funct3: U8, rd: U8, rs1: U8, imm: I32) -> Result<(), String> {
        let rs1_idx = rs1 as usize;
        let rd_idx = rd as usize;
        let result = match funct3 {
            0 => (self.x[rs1_idx] as U32).wrapping_add(imm as U32),
            1 => (self.x[rs1_idx] as U32) << (imm & 0x1f),
            5 => {
                if imm & 0x400 == 0 {
                    (self.x[rs1_idx] as U32) >> (imm & 0x1f)
                } else {
                    ((self.x[rs1_idx] as I32) >> (imm & 0x1f)) as U32
                }
            }
            _ => return Err(String::from("Unknown ALU immediate 32-bit operation")),
        };
        self.x[rd_idx] = result as I64 as U64;
        Ok(())
    }

    fn exec_alu_reg(&mut self, funct3: U8, funct7: U8, rd: U8, rs1: U8, rs2: U8) -> Result<(), String> {
        let rs1_idx = rs1 as usize;
        let rs2_idx = rs2 as usize;
        let rd_idx = rd as usize;
        let op = (funct7 as u16) << 3 | funct3 as u16;
        match op {
            0x000 => self.x[rd_idx] = self.x[rs1_idx].wrapping_add(self.x[rs2_idx]),
            0x100 => self.x[rd_idx] = self.x[rs1_idx].wrapping_sub(self.x[rs2_idx]),
            0x001 => self.x[rd_idx] = self.x[rs1_idx] << (self.x[rs2_idx] & 0x3f),
            0x002 => self.x[rd_idx] = if (self.x[rs1_idx] as I64) < (self.x[rs2_idx] as I64) { 1 } else { 0 },
            0x003 => self.x[rd_idx] = if self.x[rs1_idx] < self.x[rs2_idx] { 1 } else { 0 },
            0x004 => self.x[rd_idx] = self.x[rs1_idx] ^ self.x[rs2_idx],
            0x005 => self.x[rd_idx] = self.x[rs1_idx] >> (self.x[rs2_idx] & 0x3f),
            0x105 => self.x[rd_idx] = (self.x[rs1_idx] as I64 >> (self.x[rs2_idx] & 0x3f)) as U64,
            0x006 => self.x[rd_idx] = self.x[rs1_idx] | self.x[rs2_idx],
            0x007 => self.x[rd_idx] = self.x[rs1_idx] & self.x[rs2_idx],
            0x008 => self.x[rd_idx] = self.x[rs1_idx].wrapping_mul(self.x[rs2_idx]),
            0x009 => self.x[rd_idx] = self.mulh(self.x[rs1_idx] as I64, self.x[rs2_idx] as I64),
            0x00a => self.x[rd_idx] = self.mulhsu(self.x[rs1_idx] as I64, self.x[rs2_idx]),
            0x00b => self.x[rd_idx] = self.mulhu(self.x[rs1_idx], self.x[rs2_idx]),
            0x00c => {
                self.x[rd_idx] = if self.x[rs2_idx] != 0 {
                    if self.x[rs1_idx] as I64 == I64::MIN && self.x[rs2_idx] as I64 == -1 {
                        I64::MIN as U64
                    } else {
                        (self.x[rs1_idx] as I64 / self.x[rs2_idx] as I64) as U64
                    }
                } else {
                    U64::MAX
                };
            }
            0x00d => {
                self.x[rd_idx] = if self.x[rs2_idx] != 0 {
                    self.x[rs1_idx] / self.x[rs2_idx]
                } else {
                    U64::MAX
                };
            }
            0x00e => {
                self.x[rd_idx] = if self.x[rs2_idx] != 0 {
                    if self.x[rs1_idx] as I64 == I64::MIN && self.x[rs2_idx] as I64 == -1 {
                        0
                    } else {
                        (self.x[rs1_idx] as I64 % self.x[rs2_idx] as I64) as U64
                    }
                } else {
                    self.x[rs1_idx]
                };
            }
            0x00f => {
                self.x[rd_idx] = if self.x[rs2_idx] != 0 {
                    self.x[rs1_idx] % self.x[rs2_idx]
                } else {
                    self.x[rs1_idx]
                };
            }
            _ => return Err(String::from("Unknown ALU register operation")),
        }
        Ok(())
    }

    fn exec_alu_reg32(&mut self, funct3: U8, funct7: U8, rd: U8, rs1: U8, rs2: U8) -> Result<(), String> {
        let rs1_idx = rs1 as usize;
        let rs2_idx = rs2 as usize;
        let rd_idx = rd as usize;
        let op = (funct7 as u16) << 3 | funct3 as u16;
        let a = self.x[rs1_idx] as U32;
        let b = self.x[rs2_idx] as U32;

        let result: I32 = match op {
            0x000 => a.wrapping_add(b) as I32,
            0x100 => a.wrapping_sub(b) as I32,
            0x001 => (a << (b & 0x1f)) as I32,
            0x005 => (a >> (b & 0x1f)) as I32,
            0x105 => (a as I32) >> (b & 0x1f),
            0x008 => (a as I32).wrapping_mul(b as I32),
            0x00c => {
                if b != 0 {
                    if a as I32 == I32::MIN && b as I32 == -1 {
                        I32::MIN
                    } else {
                        (a as I32) / (b as I32)
                    }
                } else {
                    -1
                }
            }
            0x00d => {
                if b != 0 {
                    (a as u32 / b) as I32
                } else {
                    -1
                }
            }
            0x00e => {
                if b != 0 {
                    (a as I32) % (b as I32)
                } else {
                    a as I32
                }
            }
            0x00f => {
                if b != 0 {
                    (a % b) as I32
                } else {
                    a as I32
                }
            }
            _ => return Err(String::from("Unknown ALU register 32-bit operation")),
        };
        self.x[rd_idx] = result as I64 as U64;
        Ok(())
    }

    fn exec_system(&mut self, funct3: U8, rd: U8) -> Result<(), String> {
        if funct3 != 0 {
            self.handle_csr(rd);
            return Ok(());
        }

        match self.inst {
            0x00000073 => {
                self.handle_ecall();
                return Ok(());
            }
            0x00100073 => {
                let prev_inst_addr = if self.pc >= 8 { self.pc - 8 } else { 0 };
                let post_inst_addr = if self.pc + 3 < self.p_end { self.pc } else { self.p_end - 1 };
                let has_prev = self.mem_load::<U32>(prev_inst_addr) == 0x01f01013;
                let has_next = self.mem_load::<U32>(post_inst_addr) == 0x40705013;
                if has_prev && has_next {
                    self.handle_semihost();
                } else {
                    self.halted.store(true, Ordering::SeqCst);
                }
                return Ok(());
            }
            0x10500073 | 0x30200073 | 0x10200073 | 0x00200073 => {
                return Err(format!(
                    "Privilege-mode return instruction (MRET/SRET/URET, inst=0x{:x}) at pc=0x{:x}: this VM has no privilege levels",
                    self.inst,
                    self.pc - 4
                ));
            }
            _ => {
                return Err(format!(
                    "Unknown SYSTEM instruction 0x{:x} at pc=0x{:x}",
                    self.inst,
                    self.pc - 4
                ));
            }
        }
    }

    fn handle_csr(&mut self, rd: U8) {
        if rd != 0 {
            self.x[rd as usize] = 0;
        }
    }

    fn handle_semihost(&mut self) {
        panic!(
            "Semihosting call at pc=0x{:x} is not supported in this VM; implement handle_semihost() to support semihosting operations",
            self.pc - 4
        );
    }

    fn handle_ecall(&mut self) {
        panic!(
            "ECALL at pc=0x{:x} is not supported in this VM; implement handle_ecall() to support system calls",
            self.pc - 4
        );
    }

    fn mulh(&self, a: I64, b: I64) -> U64 {
        let neg = (a < 0) ^ (b < 0);
        let abs_a = a.abs() as U64;
        let abs_b = b.abs() as U64;

        let (hi, lo) = self.mulu64_128(abs_a, abs_b);

        if neg {
            let hi = !hi;
            let lo = !lo;
            if lo == 0 {
                hi.wrapping_add(1)
            } else {
                hi
            }
        } else {
            hi as I64 as U64
        }
    }

    fn mulhu(&self, a: U64, b: U64) -> U64 {
        self.mulu64_128(a, b).0
    }

    fn mulhsu(&self, a: I64, b: U64) -> U64 {
        if a < 0 {
            let abs_a = a.abs() as U64;
            let (hi, lo) = self.mulu64_128(abs_a, b);
            if lo == 0 {
                !hi.wrapping_add(1)
            } else {
                !hi
            }
        } else {
            self.mulu64_128(a as U64, b).0
        }
    }

    fn mulu64_128(&self, a: U64, b: U64) -> (U64, U64) {
        let trunc32 = 0xFFFFFFFFu64;
        let a_lo = a & trunc32;
        let a_hi = a >> 32;
        let b_lo = b & trunc32;
        let b_hi = b >> 32;

        let p0 = a_lo * b_lo;
        let p1 = a_lo * b_hi;
        let p2 = a_hi * b_lo;
        let p3 = a_hi * b_hi;

        let mid = (p0 >> 32) + (p1 & trunc32) + (p2 & trunc32);
        let lo = (p0 & trunc32) | (mid << 32);
        let hi = p3 + (p1 >> 32) + (p2 >> 32) + (mid >> 32);

        (hi, lo)
    }
}

fn main() {
    let _vm = VM::new(4096, 1024 * 1024);
}