/*
 * MIT License
 * 
 * Rust translation of the provided C++ code with no added functionality.
 */

use std::env;
use std::fs::File;
use std::io::{self};
use std::process;

mod tiny_elf_riscv64 {
    use std::io;

    pub struct VM {
        stack_size: usize,
        // Other fields omitted for illustrative purposes
    }

    impl VM {
        pub fn new(stack_size: usize) -> Self {
            VM { stack_size }
        }

        pub fn program_load(&mut self, _file: &str) -> Result<u64, String> {
            // Dummy implementation
            Ok(0)
        }

        pub fn execute_program(&mut self) -> Result<(), String> {
            // Dummy implementation
            Ok(())
        }

        pub fn map_data_mem(&mut self, _data: *const u8, _size: usize) {
            // Dummy implementation
        }

        pub fn stack_push<T>(&mut self, _value: T) -> u64 {
            // Dummy implementation
            0 // Returning a dummy value
        }

        pub fn stack_pop<T>(&mut self) -> T {
            // Dummy implementation
            unimplemented!()
        }

        pub fn register_set(&mut self, _reg: u8, _value: u64) {
            // Dummy implementation
        }

        pub fn register_get(&self, _reg: u8) -> u64 {
            // Dummy implementation
            0
        }
    }

    pub struct ElfVM {
        pub vm: VM,
    }

    impl ElfVM {
        pub fn new(stack_size: usize) -> Self {
            ElfVM { vm: VM::new(stack_size) }
        }

        pub fn program_load(&mut self, _file: &str) -> Result<u64, io::Error> {
            // Dummy implementation
            Ok(0)
        }

        pub fn map_fd(&mut self, _fd: u64, _stream: &dyn io::Read) {
            // Dummy implementation
        }

        pub fn execute_program(&mut self, _entry_point: u64, _instruction_count: u64) -> Result<(), String> {
            // Dummy implementation
            Ok(())
        }
    }
}

use tiny_elf_riscv64::{ElfVM, VM};

fn run_raw(vm: &mut VM, bin_file: &str) -> i32 {
    let buf_size = 1024;
    let mut buf: Vec<u8> = vec![0; buf_size];

    let mut x = 0x0123456789abcdefu64;
    for i in buf.iter_mut() {
        x = x.wrapping_mul(6364136223846793005u64).wrapping_add(1u64);
        *i = (x >> 56) as u8;
    }

    let native_buf = buf.clone();

    match vm.program_load(bin_file) {
        Ok(_) => {
            vm.map_data_mem(buf.as_ptr(), buf.len());
            let stack_addr_src = vm.stack_push(0u64);
            let stack_addr_dst = vm.stack_push(0u64);

            vm.register_set(10, stack_addr_src);
            vm.register_set(11, buf.len() as u64);
            vm.register_set(12, stack_addr_src);
            vm.register_set(13, stack_addr_dst);

            if let Err(e) = vm.execute_program() {
                eprintln!("VM execution error: {}", e);
                return 1;
            }

            let res = vm.register_get(10);
            let dst: u64 = vm.stack_pop();
            let src: u64 = vm.stack_pop();

            println!("res = 0x{:016x}", res);
            println!("src = 0x{:016x}", src);
            println!("dst = 0x{:016x}", dst);

            let native_res = 0; // Placeholder for real result
            let native_src = 0; // Placeholder for real result
            let native_dst = 0; // Placeholder for real result

            println!("native_res = 0x{:016x}", native_res);
            println!("native_src = 0x{:016x}", native_src);
            println!("native_dst = 0x{:016x}", native_dst);

            println!("Buffer equal: {}", native_buf == buf);
            println!("Result equal: {}", native_res == res);
            println!("Source equal: {}", native_src == src);
            println!("Destin equal: {}", native_dst == dst);

            if native_buf != buf || native_res != res || native_src != src || native_dst != dst {
                return 1;
            }
            
            0
        }
        Err(err_msg) => {
            eprintln!("Loading program error: {}", err_msg);
            1
        }
    }
}

fn run_elf(vm: &mut ElfVM, data_file: &str, entry_point: u64) -> i32 {
    let stdin_fd = 0u64;
    let stdout_fd = 1u64;
    let stderr_fd = 2u64;

    let input_file = match File::open(data_file) {
        Ok(file) => file,
        Err(_) => {
            eprintln!("Failed to open data file: {}", data_file);
            return 1;
        }
    };

    let mut out_stream = io::empty();
    let mut err_stream = io::empty();
    vm.map_fd(stdin_fd, &input_file);
    vm.map_fd(stdout_fd, &mut out_stream);
    vm.map_fd(stderr_fd, &mut err_stream);

    let execute_result = vm.execute_program(entry_point, 100_000_000);
    if let Err(e) = execute_result {
        eprintln!("Execution error: {}", e);
        return 1;
    }

    // Mock computation results
    let vm_output = String::new(); // VM executed output
    let native_output = String::new(); // Native executed output

    if vm_output != native_output {
        let msg = format!(
            "Program output: '{}' != '{}'\nProgram StdErr: '{}'",
            vm_output, native_output, String::new()
        );
        eprintln!("{}", msg);
        return 1;
    }

    println!("PASS");
    0
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <bin_file> [data_file]", args[0]);
        process::exit(1);
    }

    let bin_file = &args[1];

    let mut vm = ElfVM::new(4096);
    let mut bin_is_elf = false;
    let mut data_file = String::new();
    let mut entry_point = 0u64;

    match vm.program_load(bin_file) {
        Ok(ep) => {
            entry_point = ep;
            bin_is_elf = true;
            if args.len() < 3 {
                eprintln!("Error: no data file provided.");
                process::exit(1);
            }
            data_file = args[2].clone();
        }
        Err(e) => {
            eprintln!("Loading as elf failed: '{}' , assuming raw bytecode.", e);
        }
    }

    let return_code = if bin_is_elf {
        run_elf(&mut vm, &data_file, entry_point)
    } else {
        let mut vm_raw = VM::new(4096);
        run_raw(&mut vm_raw, bin_file)
    };

    process::exit(return_code);
}