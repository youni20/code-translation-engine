use std::fs::File;
use std::io::{self, Read};
use std::sync::atomic::{AtomicBool, Ordering};

pub struct VM {
    pc: u64,                       // Program counter
    inst: u32,                     // Current instruction
    program: Vec<u8>,              // Program memory
    x: [u64; 32],                  // Registers x0-x31
    stack: Vec<u8>,                // Stack memory
    data: Vec<u8>,                 // Data memory
    halted: AtomicBool,            // Program exited or externally halted
    max_prog_size: usize,          // Maximum allowed program image size (bytes)

    // Virtual addressing:
    p_beg: u64,   // Program mem begin
    p_end: u64,   // Program mem end
    d_beg: u64,   // Data mem begin
    d_end: u64,   // Data mem end
    s_beg: u64,   // Stack mem begin
    s_end: u64,   // Stack mem end
}

impl VM {
    pub fn new(stack_size: usize, max_program_size: usize) -> VM {
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

    pub fn program_load(&mut self, prog_filename: &str) -> io::Result<u64> {
        self.program = self.load_program(prog_filename)?;
        self.reset();
        Ok(self.p_beg)
    }

    pub fn program_load_from_bytes(&mut self, prog: &[u8]) -> Result<u64, String> {
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

    pub fn stack_push<T>(&mut self, val: T) -> u64 {
        let size = std::mem::size_of::<T>();
        self.x[2] -= size as u64;
        self.mem_store(self.x[2], val);
        self.x[2]
    }

    pub fn stack_pop<T>(&mut self) -> T {
        let size = std::mem::size_of::<T>();
        self.x[2] += size as u64;
        self.mem_load::<T>(self.x[2] - size as u64)
    }

    pub fn stack_peek<T>(&mut self) -> T {
        self.mem_load::<T>(self.x[2])
    }

    pub fn execute_program(&mut self, entry_point: u64) -> Result<(), String> {
        let prog_sz = self.program.len() as u64;
        let sentinel_pc = (prog_sz + 3) & !3u64;
        self.pc = entry_point;
        self.halted.store(false, Ordering::SeqCst);
        let mut count = 0;

        if prog_sz < 4 {
            return Err("Program too small (must be at least 4 bytes)".to_string());
        }

        while !self.halted.load(Ordering::SeqCst) {
            if self.pc > prog_sz - 4 {
                return Err("PC jumped program region".to_string());
            }

            if count > 100000 {
                return Err("Maximum instruction count exceeded".to_string());
            }

            self.execute_instruction();
            count += 1;

            if self.pc == sentinel_pc {
                self.halted.store(true, Ordering::SeqCst);
            }
        }
        Ok(())
    }

    pub fn halt_program(&mut self) -> bool {
        !self.halted.swap(true, Ordering::SeqCst)
    }

    pub fn reset(&mut self) {
        self.x.iter_mut().for_each(|xn| *xn = 0);
        self.x[1] = (self.program.len() + 3) as u64 & !3;
        self.x[2] = (self.program.len() + 64 + self.data.len() + 64 + self.stack.len()) as u64;
        self.x[8] = self.x[2];

        self.p_end = self.program.len() as u64;
        self.d_beg = self.program.len() as u64 + 64;
        self.d_end = self.program.len() as u64 + 64 + self.data.len() as u64;
        self.s_beg = self.program.len() as u64 + 64 + self.data.len() as u64 + 64;
        self.s_end = self.program.len() as u64 + 64 + self.data.len() as u64 + 64 + self.stack.len() as u64;
    }

    fn load_program(&self, filename: &str) -> io::Result<Vec<u8>> {
        let mut file = File::open(filename)?;
        let metadata = file.metadata()?;
        let file_size = metadata.len() as usize;

        if file_size > self.max_prog_size {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("Program too large (max {})", self.max_prog_size)));
        }

        let mut buffer = vec![0; file_size];
        file.read(&mut buffer)?;
        Ok(buffer)
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
        (((self.inst as i32) & 0x80000000) as i64 >> 19)
            | (((self.inst >> 7) & 0x1e) as i64)
            | (((self.inst >> 20) & 0x7e0) as i64)
            | (((self.inst & 0x80) << 4) as i64)
    }

    fn imm_j(&self) -> i64 {
        (((self.inst as i32) & 0x80000000) as i64 >> 11)
            | (((self.inst & 0xff000) as i64))
            | (((self.inst >> 9) & 0x800) as i64)
            | (((self.inst >> 20) & 0x7fe) as i64)
    }

    fn imm_u(&self) -> u64 {
        (self.inst & 0xfffff000) as u64
    }

    fn execute_instruction(&mut self) {
        self.inst = u32::from_le_bytes(self.program[self.pc as usize..(self.pc as usize + 4)].try_into().unwrap());
        self.pc += 4;
        self.x[0] = 0; // Ensure x0 stays zero

        match self.opcode() {
            0x37 => self.x[self.rd() as usize] = self.imm_u(),               // LUI
            0x17 => self.x[self.rd() as usize] = self.pc.wrapping_sub(4) + self.imm_u(), // AUIPC
            0x6f => {
                self.x[self.rd() as usize] = self.pc;
                self.pc = self.pc.wrapping_add(self.imm_j() as u64).wrapping_sub(4); // JAL
            }
            0x67 => {
                let target = (self.x[self.rs1() as usize] as i64).wrapping_add(self.imm_i()) as u64 & !1;
                self.x[self.rd() as usize] = self.pc;
                self.pc = target;
            } // JALR
            0x63 => self.exec_branch(self.funct3(), self.rs1(), self.rs2(), self.imm_b()), // Branch
            0x03 => self.exec_load(self.funct3(), self.rd(), self.rs1(), self.imm_i()), // Load
            0x23 => self.exec_store(self.funct3(), self.rs1(), self.rs2(), self.imm_s()), // Store
            0x13 => self.exec_alu_imm(self.funct3(), self.rd(), self.rs1(), self.imm_i()), // ALU immediate
            0x1b => self.exec_alu_imm32(self.funct3(), self.rd(), self.rs1(), self.imm_i() as i32), // ALU immediate 32-bit
            0x33 => self.exec_alu_reg(self.funct3(), self.funct7(), self.rd(), self.rs1(), self.rs2()), // ALU register
            0x3b => self.exec_alu_reg32(self.funct3(), self.funct7(), self.rd(), self.rs1(), self.rs2()), // ALU register 32-bit
            0x0f => {} // FENCE (nop)
            0x73 => self.exec_system(self.funct3()), // SYSTEM
            _ => panic!("Unknown opcode"),
        }
    }

    fn mem_ptr<T>(&mut self, addr: u64) -> *mut u8 {
        const ADDR_LIMIT: u64 = 0xFFFFFFFFFFFFFFF0;
        if addr >= ADDR_LIMIT {
            panic!("Memory access out of bounds");
        }
        let addr_max = addr + std::mem::size_of::<T>() as u64 - 1;

        if addr_max < self.p_end {
            &mut self.program[addr as usize] as *mut u8
        } else if addr >= self.d_beg && addr_max < self.d_end {
            &mut self.data[(addr - self.d_beg) as usize] as *mut u8
        } else if addr >= self.s_beg && addr_max < self.s_end {
            &mut self.stack[(addr - self.s_beg) as usize] as *mut u8
        } else {
            panic!("Memory access out of bounds");
        }
    }

    fn mem_load<T>(&mut self, addr: u64) -> T {
        let mut value: T = unsafe { std::mem::zeroed() };
        unsafe {
            let dst: *mut u8 = &mut value as *mut T as *mut u8;
            std::ptr::copy_nonoverlapping(self.mem_ptr::<T>(addr), dst, std::mem::size_of::<T>());
        }
        value
    }

    fn mem_store<T>(&mut self, addr: u64, value: T) {
        unsafe {
            let src: *const u8 = &value as *const T as *const u8;
            std::ptr::copy_nonoverlapping(src, self.mem_ptr::<T>(addr), std::mem::size_of::<T>());
        }
    }

    fn exec_branch(&mut self, funct3: u8, rs1: u8, rs2: u8, imm: i64) {
        let taken = match funct3 {
            0 => self.x[rs1 as usize] == self.x[rs2 as usize],                          // BEQ
            1 => self.x[rs1 as usize] != self.x[rs2 as usize],                          // BNE
            4 => (self.x[rs1 as usize] as i64) < (self.x[rs2 as usize] as i64),         // BLT
            5 => (self.x[rs1 as usize] as i64) >= (self.x[rs2 as usize] as i64),        // BGE
            6 => self.x[rs1 as usize] < self.x[rs2 as usize],                           // BLTU
            7 => self.x[rs1 as usize] >= self.x[rs2 as usize],                          // BGEU
            _ => panic!("Unknown branch operation"),
        };
        if taken {
            self.pc = self.pc.wrapping_add(imm as u64).wrapping_sub(4);
        }
    }

    fn exec_load(&mut self, funct3: u8, rd: u8, rs1: u8, imm: i64) {
        let addr = self.x[rs1 as usize].wrapping_add(imm as u64);
        self.x[rd as usize] = match funct3 {
            0 => self.mem_load::<i8>(addr) as i64 as u64,  // LB
            1 => self.mem_load::<i16>(addr) as i64 as u64, // LH
            2 => self.mem_load::<i32>(addr) as i64 as u64, // LW
            3 => self.mem_load::<u64>(addr),                // LD
            4 => self.mem_load::<u8>(addr) as u64,         // LBU
            5 => self.mem_load::<u16>(addr) as u64,        // LHU
            6 => self.mem_load::<u32>(addr) as u64,        // LWU
            _ => panic!("Unknown load operation"),
        };
    }

    fn exec_store(&mut self, funct3: u8, rs1: u8, rs2: u8, imm: i64) {
        let addr = self.x[rs1 as usize].wrapping_add(imm as u64);
        match funct3 {
            0 => self.mem_store::<u8>(addr, self.x[rs2 as usize] as u8), // SB
            1 => self.mem_store::<u16>(addr, self.x[rs2 as usize] as u16), // SH
            2 => self.mem_store::<u32>(addr, self.x[rs2 as usize] as u32), // SW
            3 => self.mem_store::<u64>(addr, self.x[rs2 as usize]),       // SD
            _ => panic!("Unknown store operation"),
        }
    }

    fn exec_alu_imm(&mut self, funct3: u8, rd: u8, rs1: u8, imm: i64) {
        self.x[rd as usize] = match funct3 {
            0 => self.x[rs1 as usize].wrapping_add(imm as u64),              // ADDI
            1 => self.x[rs1 as usize] << (imm as u64 & 0x3f), // SLLI
            2 => ((self.x[rs1 as usize] as i64) < imm) as u64, // SLTI
            3 => (self.x[rs1 as usize] < imm as u64) as u64,   // SLTIU
            4 => self.x[rs1 as usize] ^ imm as u64,            // XORI
            5 => if (imm & 0x400) == 0 {
                    self.x[rs1 as usize] >> (imm as u64 & 0x3f) // SRLI
                } else {
                    ((self.x[rs1 as usize] as i64) >> (imm as u64 & 0x3f)) as u64 // SRAI
                },
            6 => self.x[rs1 as usize] | imm as u64, // ORI
            7 => self.x[rs1 as usize] & imm as u64, // ANDI
            _ => panic!("Unknown alu_imm operation"),
        };
    }

    fn exec_alu_imm32(&mut self, funct3: u8, rd: u8, rs1: u8, imm: i32) {
        let result = match funct3 {
            0 => (self.x[rs1 as usize] as u32).wrapping_add(imm as u32),           // ADDIW
            1 => (self.x[rs1 as usize] as u32) << (imm & 0x1f),    // SLLIW
            5 => if (imm & 0x400) == 0 {
                    (self.x[rs1 as usize] as u32) >> (imm & 0x1f) // SRLIW
                } else {
                    ((self.x[rs1 as usize] as i32) >> (imm & 0x1f)) as u32 // SRAIW
                },
            _ => panic!("Unknown alu_imm32 operation"),
        };

        self.x[rd as usize] = result as i32 as i64; // Sign-extend
    }

    fn exec_alu_reg(&mut self, funct3: u8, funct7: u8, rd: u8, rs1: u8, rs2: u8) {
        let op = (funct7 << 3) | funct3;
        match op {
            0x00 => self.x[rd as usize] = self.x[rs1 as usize].wrapping_add(self.x[rs2 as usize]), // ADD
            0x100 => self.x[rd as usize] = self.x[rs1 as usize].wrapping_sub(self.x[rs2 as usize]), // SUB
            0x01 => self.x[rd as usize] = self.x[rs1 as usize] << (self.x[rs2 as usize] & 0x3f), // SLL
            0x02 => self.x[rd as usize] = ((self.x[rs1 as usize] as i64) < (self.x[rs2 as usize] as i64)) as u64, // SLT
            0x03 => self.x[rd as usize] = (self.x[rs1 as usize] < self.x[rs2 as usize]) as u64, // SLTU
            0x04 => self.x[rd as usize] = self.x[rs1 as usize] ^ self.x[rs2 as usize], // XOR
            0x05 => self.x[rd as usize] = self.x[rs1 as usize] >> (self.x[rs2 as usize] & 0x3f), // SRL
            0x105 => self.x[rd as usize] = ((self.x[rs1 as usize] as i64) >> (self.x[rs2 as usize] & 0x3f)) as u64, // SRA
            0x06 => self.x[rd as usize] = self.x[rs1 as usize] | self.x[rs2 as usize], // OR
            0x07 => self.x[rd as usize] = self.x[rs1 as usize] & self.x[rs2 as usize], // AND

            // M extension
            0x08 => self.x[rd as usize] = self.x[rs1 as usize].wrapping_mul(self.x[rs2 as usize]), // MUL
            0x09 => self.x[rd as usize] = self.mulh(self.x[rs1 as usize] as i64, self.x[rs2 as usize] as i64), // MULH
            0x0A => self.x[rd as usize] = self.mulhsu(self.x[rs1 as usize] as i64, self.x[rs2 as usize]), // MULHSU
            0x0B => self.x[rd as usize] = self.mulhu(self.x[rs1 as usize], self.x[rs2 as usize]), // MULHU
            0x0C => { // DIV
                self.x[rd as usize] = if self.x[rs2 as usize] != 0 {
                    if (self.x[rs1 as usize] as i64) == i64::MIN && self.x[rs2 as usize] as i64 == -1 {
                        i64::MIN as u64
                    } else {
                        (self.x[rs1 as usize] as i64).wrapping_div(self.x[rs2 as usize] as i64) as u64
                    }
                } else {
                    u64::MAX
                };
            }
            0x0D => { // DIVU
                self.x[rd as usize] = if self.x[rs2 as usize] != 0 {
                    self.x[rs1 as usize].wrapping_div(self.x[rs2 as usize])
                } else {
                    u64::MAX
                };
            }
            0x0E => { // REM
                self.x[rd as usize] = if self.x[rs2 as usize] != 0 {
                    if (self.x[rs1 as usize] as i64) == i64::MIN && self.x[rs2 as usize] as i64 == -1 {
                        0
                    } else {
                        (self.x[rs1 as usize] as i64).wrapping_rem(self.x[rs2 as usize] as i64) as u64
                    }
                } else {
                    self.x[rs1 as usize]
                };
            }
            0x0F => { // REMU
                self.x[rd as usize] = if self.x[rs2 as usize] != 0 {
                    self.x[rs1 as usize].wrapping_rem(self.x[rs2 as usize])
                } else {
                    self.x[rs1 as usize]
                };
            }
            _ => panic!("Unknown alu_reg operation"),
        }
    }

    fn exec_alu_reg32(&mut self, funct3: u8, funct7: u8, rd: u8, rs1: u8, rs2: u8) {
        let op = (funct7 << 3) | funct3;
        let result = match op {
            0x00 => ((self.x[rs1 as usize] as u32).wrapping_add(self.x[rs2 as usize] as u32)) as i32, // ADDW
            0x100 => ((self.x[rs1 as usize] as u32).wrapping_sub(self.x[rs2 as usize] as u32)) as i32, // SUBW
            0x01 => ((self.x[rs1 as usize] as u32) << (self.x[rs2 as usize] as u32 & 0x1f)) as i32, // SLLW
            0x05 => ((self.x[rs1 as usize] as u32) >> (self.x[rs2 as usize] as u32 & 0x1f)) as i32, // SRLW
            0x105 => ((self.x[rs1 as usize] as i32) >> (self.x[rs2 as usize] as u32 & 0x1f)) as i32, // SRAW

            // M extension 32-bit
            0x08 => ((self.x[rs1 as usize] as i32).wrapping_mul(self.x[rs2 as usize] as i32)) as i32, // MULW

            0x0C => { // DIVW
                if self.x[rs2 as usize] != 0 {
                    if (self.x[rs1 as usize] as i32) == i32::MIN && self.x[rs2 as usize] as i32 == -1 {
                        i32::MIN
                    } else {
                        (self.x[rs1 as usize] as i32).wrapping_div(self.x[rs2 as usize] as i32)
                    }
                } else {
                    -1
                }
            }

            0x0D => { // DIVUW
                if self.x[rs2 as usize] != 0 {
                    (self.x[rs1 as usize] as u32).wrapping_div(self.x[rs2 as usize] as u32) as i32
                } else {
                    -1
                }
            }

            0x0E => { // REMW
                if self.x[rs2 as usize] != 0 {
                    (self.x[rs1 as usize] as i32).wrapping_rem(self.x[rs2 as usize] as i32)
                } else {
                    self.x[rs1 as usize] as i32
                }
            }

            0x0F => { // REMUW
                if self.x[rs2 as usize] != 0 {
                    (self.x[rs1 as usize] as u32).wrapping_rem(self.x[rs2 as usize] as u32) as i32
                } else {
                    self.x[rs1 as usize] as i32
                }
            }

            _ => panic!("Unknown alu_reg32 operation"),
        };

        self.x[rd as usize] = result as i64; // Sign-extend to 64 bits
    }

    fn exec_system(&mut self, funct3: u8) {
        if funct3 != 0 {
            self.handle_csr();
            return;
        }

        match self.inst {
            0x00000073 => self.handle_ecall(), // ECALL
            0x00100073 => { // EBREAK
                let has_prev = if self.pc >= 8 {
                    self.mem_load::<u32>(self.pc - 8) == 0x01f01013u32
                } else {
                    false
                };

                let has_next = if self.pc + 3 < self.p_end {
                    self.mem_load::<u32>(self.pc) == 0x40705013u32
                } else {
                    false
                };

                if has_prev && has_next {
                    self.handle_semihost();
                } else {
                    self.halted.store(true, Ordering::SeqCst);
                }
            }

            _ => panic!(
                "Unknown SYSTEM instruction 0x{:x} at pc=0x{:x}",
                self.inst,
                self.pc.wrapping_sub(4)
            ),
        }
    }

    fn handle_csr(&mut self) {
        let d = self.rd();

        if d != 0 {
            self.x[d as usize] = 0;
        }
    }

    fn handle_semihost(&self) {
        panic!(
            "Semihosting call at pc=0x{:x} is not supported in this VM",
            self.pc.wrapping_sub(4)
        )
    }

    fn handle_ecall(&self) {
        panic!(
            "ECALL at pc=0x{:x} is not supported in this VM",
            self.pc.wrapping_sub(4)
        )
    }

    fn mulh(&self, a: i64, b: i64) -> u64 {
        let result = (a as i128) * (b as i128);
        (result >> 64) as u64
    }

    fn mulhu(&self, a: u64, b: u64) -> u64 {
        let result = (a as u128) * (b as u128);
        (result >> 64) as u64
    }

    fn mulhsu(&self, a: i64, b: u64) -> u64 {
        let result = (a as i128) * (b as i128);
        (result >> 64) as u64
    }
}

fn main() {
    // Placeholder for main function since it's required for compilation.
    println!("TinyRISCV64 VM");
}