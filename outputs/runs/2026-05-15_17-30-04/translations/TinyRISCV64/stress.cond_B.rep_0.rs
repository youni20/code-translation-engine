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

use std::env;

pub mod tiny_riscv64 {
    pub struct VM;

    impl VM {
        pub fn new(_stack_size: usize) -> Self {
            VM
        }

        pub fn program_load(&self, _file: &str) -> Result<u64, &'static str> {
            // Simulate loading a binary program
            Ok(0)
        }

        pub fn map_data_mem(&self, _data: &[u8]) -> u64 {
            // Simulate mapping data to memory
            0
        }

        pub fn stack_push<T>(&self, _value: T) -> u64 {
            // Simulate push on stack (return random address)
            0
        }

        pub fn stack_pop<T: Default>(&self) -> T {
            // Simulate pop from stack (return dummy data)
            T::default()
        }

        pub fn register_set(&self, _reg: u8, _value: u64) {
            // Set value in VM register
        }

        pub fn register_get(&self, _reg: u8) -> u64 {
            // Get value from VM register
            0
        }

        pub fn execute_program(&self) {
            // Execute program
        }
    }
}

fn run_raw(vm: &tiny_riscv64::VM, bin_file: &str) -> i32 {
    let buf_size = 1024;
    let mut buf = vec![0u8; buf_size];

    let mut x: u64 = 0x0123456789abcdef;
    for byte in &mut buf {
        x = x.wrapping_mul(6364136223846793005).wrapping_add(1);
        *byte = (x >> 56) as u8;
    }

    let native_buf = buf.clone();

    vm.program_load(bin_file).unwrap();
    let data_addr_buf = vm.map_data_mem(&buf);

    let stack_addr_src = vm.stack_push(0u64);
    let stack_addr_dst = vm.stack_push(0u64);

    vm.register_set(10, data_addr_buf);
    vm.register_set(11, buf.len() as u64);
    vm.register_set(12, stack_addr_src);
    vm.register_set(13, stack_addr_dst);

    vm.execute_program();

    let res = vm.register_get(10);
    let dst = vm.stack_pop::<u64>();
    let src = vm.stack_pop::<u64>();

    println!("res = 0x{:016x}", res);
    println!("src = 0x{:016x}", src);
    println!("dst = 0x{:016x}", dst);

    // Replace with actual native calculations
    let native_src = 0u64;
    let native_dst = 0u64;
    let native_res = 0u64;

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

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <bin_file> [data_file]", args[0]);
        std::process::exit(1);
    }

    let bin_file = &args[1];
    let vm = tiny_riscv64::VM::new(4096);

    let result = run_raw(&vm, bin_file);
    std::process::exit(result);
}