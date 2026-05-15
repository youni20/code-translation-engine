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

use std::collections::VecDeque;
use std::env;
use std::fs::File;
use std::io::Read;
use std::process;
use std::error::Error;

mod tinyriscv64 {
    use std::error::Error;

    pub struct VM {
        stack: Vec<u64>,
        // Additional fields omitted for brevity
    }

    impl VM {
        pub fn new(stack_size: usize) -> Self {
            Self {
                stack: Vec::with_capacity(stack_size / 8), // Assuming 8-byte elements
                // Initialize other fields
            }
        }

        pub fn program_load(&mut self, _filename: &str) -> Result<(), Box<dyn Error>> {
            // Method implementation goes here
            Ok(())
        }

        pub fn register_get(&self, _reg: u32) -> u64 {
            // Placeholder implementation
            0
        }

        pub fn execute_program(&mut self) {
            // Method implementation goes here
        }

        pub fn stack_pop(&mut self) -> u64 {
            self.stack.pop().expect("Stack underflow")
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <asm_file> <bin_file> [all|failed]", args[0]);
        process::exit(1);
    }

    let asm_file = &args[1];
    let bin_file = &args[2];
    let print_all = args.get(3).map_or(false, |arg| arg == "all");

    let mut stack_values = VecDeque::new();

    let mut vm = tinyriscv64::VM::new(4096);
    vm.program_load(bin_file)?;

    let sp_before = vm.register_get(2);
    vm.execute_program();

    while vm.register_get(2) < sp_before {
        stack_values.push_front(vm.stack_pop());
    }

    let mut file = File::open(asm_file)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    let test_block_regex = r"(\# TEST:[\s\S]*?EXPECTED PUSH:\s*(0x[0-9A-Fa-f]+)[\s\S]*?)(?=\#\s*TEST|$)";
    let mut test_cases = Vec::new();
    
    for caps in content.split("\n").collect::<Vec<&str>>().chunks(2) {
        if let Some(block) = caps.get(0) {
            if let Some(&expected_str) = caps.get(1) {
                if let Some(expected_value) = expected_str.split_whitespace().last() {
                    if let Ok(expected_val) = u64::from_str_radix(&expected_value[2..], 16) {
                        test_cases.push((block.to_string(), expected_val));
                    }
                }
            }
        }
    }

    let mut passed = 0;
    let mut failed = 0;

    for (i, ((_, expected), actual)) in test_cases.iter().zip(stack_values.iter()).enumerate() {
        let pass = expected == actual;
        if pass {
            passed += 1;
        } else {
            failed += 1;
        }

        if !pass || print_all {
            println!(
                "{} Test {}:\nExpected: 0x{:016X}\nActual:   0x{:016X}\n{}\n",
                if pass { "PASS" } else { "FAIL" },
                i + 1,
                expected,
                actual,
                test_cases[i].0
            );
        }
    }

    println!("Passed: {}, Failed: {}", passed, failed);
    process::exit(failed);
}