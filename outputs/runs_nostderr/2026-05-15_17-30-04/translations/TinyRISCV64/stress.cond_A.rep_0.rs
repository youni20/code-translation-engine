/*
 * MIT License
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
use std::io::{self, Read, Write};

mod tinyriscv64 {
    use std::collections::HashMap;
    use std::io::Read;

    pub struct VM {
        stack: Vec<u64>,
        registers: [u64; 32],
        memory: HashMap<u64, Vec<u8>>,
    }

    impl VM {
        pub fn new(stack_size: usize) -> Self {
            VM {
                stack: vec![0; stack_size / 8],
                registers: [0; 32],
                memory: HashMap::new(),
            }
        }

        pub fn program_load(&mut self, _bin_file: &str) -> Result<u64, String> {
            // Placeholder for actual implementation
            Ok(0)
        }

        pub fn map_data_mem(&mut self, _data: &[u8]) -> u64 {
            // Placeholder for actual implementation
            0
        }

        pub fn map_fd(&mut self, _fd: u64, _stream: Box<dyn Read + Send + Sync>) {
            // Placeholder for actual implementation
        }

        pub fn execute_program(&mut self, _entry_point: u64, _max_instructions: u64) {
            // Placeholder for actual implementation
        }

        pub fn stack_push<T>(&mut self, _value: T) -> u64 {
            // Placeholder for actual implementation
            0
        }

        pub fn stack_pop<T>(&mut self) -> T {
            // Placeholder for actual implementation
            unimplemented!()
        }

        pub fn register_set(&mut self, _index: usize, _value: u64) {
            // Placeholder for actual implementation
        }

        pub fn register_get(&self, _index: usize) -> u64 {
            // Placeholder for actual implementation
            0
        }
    }

    pub struct ElfVM;

    impl ElfVM {
        pub fn new(_stack_size: usize) -> Self {
            ElfVM
        }

        pub fn program_load(&mut self, _bin_file: &str) -> Result<u64, String> {
            // Placeholder for actual implementation
            Ok(0)
        }

        pub fn map_fd(&mut self, _fd: u64, _stream: Box<dyn Read + Send + Sync>) {
            // Placeholder for actual implementation
        }

        pub fn execute_program(&mut self, _entry_point: u64, _max_instructions: u64) {
            // Placeholder for actual implementation
        }
    }
}

fn run_elf(vm: &mut tinyriscv64::ElfVM, data_file: &str, entry_point: u64) -> Result<i32, String> {
    let stdin_fd = 0;
    let stdout_fd = 1;
    let stderr_fd = 2;

    // Map input fd
    let pDataStream = Box::new(File::open(data_file).map_err(|_| format!("Failed to open data file: {}", data_file))?) as Box<dyn Read + Send + Sync>;
    vm.map_fd(stdin_fd, pDataStream);

    // Placeholder streams
    let pOutStream = Box::new(io::Cursor::new(Vec::new())) as Box<dyn Read + Send + Sync>;
    let pErrStream = Box::new(io::Cursor::new(Vec::new())) as Box<dyn Read + Send + Sync>;

    vm.map_fd(stdout_fd, pOutStream);
    vm.map_fd(stderr_fd, pErrStream);

    vm.execute_program(entry_point, 100 * 1024 * 1024); // 100 million instructions max

    // Placeholder for further logic
    Ok(0)
}

fn run_raw(_vm: &mut tinyriscv64::VM, _bin_file: &str) -> Result<i32, String> {
    // Placeholder for actual logic
    Ok(0)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <bin_file> [data_file]", args[0]);
        return Ok(());
    }

    let bin_file = &args[1];
    let mut vm = tinyriscv64::ElfVM::new(4096);
    let bin_is_elf: bool;
    let data_file: Option<&str>;
    let entry_point: u64;

    if let Ok(ep) = vm.program_load(bin_file) {
        entry_point = ep;
        bin_is_elf = true;
        if args.len() < 3 {
            eprintln!("Error: no data file provided.");
            return Ok(());
        }
        data_file = Some(&args[2]);
    } else {
        eprintln!("Loading as elf failed, assuming raw bytecode.");
        bin_is_elf = false;
        data_file = None;
        entry_point = 0; // Assign a default initialization to entry_point
    }

    let result = match bin_is_elf {
        true => run_elf(&mut vm, data_file.unwrap(), entry_point)?,
        false => {
            let mut raw_vm = tinyriscv64::VM::new(4096);
            run_raw(&mut raw_vm, bin_file)?
        }
    };

    std::process::exit(result);
}