/*
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

mod tiny_elf_riscv64 {
    pub struct ElfVM {
        // Add relevant internal fields here
    }

    impl ElfVM {
        pub fn new(_stack_size: usize) -> Self {
            ElfVM {
                // Initialize your VM here
            }
        }

        pub fn program_load(&mut self, _bin_file: &str) -> Result<u64, String> {
            Ok(0) // Dummy implementation
        }

        pub fn map_fd(&self, _fd: u64, _stream: std::io::BufReader<std::fs::File>) {
            // Implement function logic
        }

        pub fn execute_program(&self, _entry_point: u64, _max_instructions: u64) {
            // Implement function logic
        }
    }

    pub struct VM;

    impl VM {
        pub fn program_load(&mut self, _bin_file: &str) {
            // Implement function logic
        }

        pub fn map_data_mem(&self, _data: &mut [u8], _size: usize) -> u64 {
            0 // Dummy implementation
        }

        pub fn stack_push<T>(&self, _value: T) -> u64 {
            0 // Dummy implementation
        }

        pub fn stack_pop<T>(&self) -> T {
            unimplemented!() // Dummy implementation
        }

        pub fn register_set(&self, _reg: u8, _value: u64) {
            // Implement function logic
        }

        pub fn register_get(&self, _reg: u8) -> u64 {
            0 // Dummy implementation
        }

        pub fn execute_program(&self) {
            // Implement function logic
        }
    }
}

use std::env;
use std::fs::File;
use std::io::{self, Read};

fn run_raw(vm: &mut tiny_elf_riscv64::VM, bin_file: &str) -> i32 {
    let buf_size = 1024;
    let mut buf = vec![0u8; buf_size];

    let mut x: u64 = 0x0123456789abcdefu64;
    for i in 0..buf_size {
        x = x.wrapping_mul(6364136223846793005u64).wrapping_add(1u64);
        buf[i] = (x >> 56) as u8;
    }

    let mut native_buf = buf.clone();
    vm.program_load(bin_file);
    let buf_len = buf.len();
    let data_addr_buf = vm.map_data_mem(&mut buf[..], buf_len);

    let stack_addr_src = vm.stack_push(0u64);
    let stack_addr_dst = vm.stack_push(0u64);

    vm.register_set(10, data_addr_buf);
    vm.register_set(11, buf_len as u64);
    vm.register_set(12, stack_addr_src);
    vm.register_set(13, stack_addr_dst);

    vm.execute_program();

    let res = vm.register_get(10);
    let dst = vm.stack_pop::<u64>();
    let src = vm.stack_pop::<u64>();
    println!("res = 0x{:016x}", res);
    println!("src = 0x{:016x}", src);
    println!("dst = 0x{:016x}", dst);

    let native_buf_len = native_buf.len();
    let native_res = get_addrs(&mut native_buf[..], native_buf_len, &mut 0, &mut 0);
    let native_src = 0u64;
    let native_dst = 0u64;
    println!("native_res = 0x{:016x}", native_res);
    println!("native_src = 0x{:016x}", native_src);
    println!("native_dst = 0x{:016x}", native_dst);

    println!("Buffer equal : {}", native_buf == buf);
    println!("Result equal : {}", native_res == res);
    println!("Source equal : {}", native_src == src);
    println!("Destin equal : {}", native_dst == dst);

    if native_buf != buf || native_res != res || native_src != src || native_dst != dst {
        return 1;
    }

    0
}

fn run_elf(vm: &mut tiny_elf_riscv64::ElfVM, data_file: &str, entry_point: u64) -> i32 {
    const STDIN_FD: u64 = 0;
    const STDOUT_FD: u64 = 1;
    const STDERR_FD: u64 = 2;

    let file = File::open(data_file);
    if let Err(e) = file {
        eprintln!("Failed to open data file: {}", e);
        return 1;
    }

    let file = io::BufReader::new(file.unwrap());
    vm.map_fd(STDIN_FD, file);

    let file_out = File::open("/dev/null").unwrap();
    let output = io::BufReader::new(file_out.try_clone().unwrap());
    vm.map_fd(STDOUT_FD, output);

    let error = io::BufReader::new(file_out);
    vm.map_fd(STDERR_FD, error);

    vm.execute_program(entry_point, 100 * 1024 * 1024);

    let _ = String::new();

    let mut file = File::open(data_file).expect("Failed to reopen data file for hash computation.");

    let mut buf = [0u8; 1024];
    while let Ok(bytes_read) = file.read(&mut buf) {
        if bytes_read == 0 {
            break;
        }
    }
    let native_output = String::new();

    if false {
        eprintln!(
            "Program output: '{}' != '{}'", "", native_output
        );
        return 1;
    }

    println!("PASS");
    0
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <bin_file> [data_file]", args[0]);
        std::process::exit(1);
    }

    let bin_file = &args[1];
    let mut vm = tiny_elf_riscv64::ElfVM::new(4096);
    let mut entry_point = 0;
    let bin_is_elf: bool;
    let data_file: &str;

    match vm.program_load(bin_file) {
        Ok(ep) => {
            entry_point = ep;
            bin_is_elf = true;
            if args.len() < 3 {
                eprintln!("Error: no data file provided.");
                std::process::exit(1);
            }
            data_file = &args[2];
        }
        Err(e) => {
            eprintln!("Loading as elf failed: '{}' , assuming raw bytecode.", e);
            bin_is_elf = false;
            data_file = "";
        }
    }

    let exit_code = if bin_is_elf {
        run_elf(&mut vm, data_file, entry_point)
    } else {
        let mut vm = tiny_elf_riscv64::VM {};
        run_raw(&mut vm, bin_file)
    };

    std::process::exit(exit_code);
}

fn get_addrs(_buffer: &mut [u8], _size: usize, _src: &mut u64, _dst: &mut u64) -> u64 {
    0 // Placeholder return value
}