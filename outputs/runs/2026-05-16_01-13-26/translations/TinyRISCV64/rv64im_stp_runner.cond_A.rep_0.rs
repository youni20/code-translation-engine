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
use std::fs;
use std::process;
use std::panic;
use regex::Regex;

mod tiny_riscv64 {
    // Assuming that TinyRISCV64 is already translated into a Rust module
    pub struct VM {
        // Define VM structure with necessary fields
        pub stack: Vec<u64>,
    }

    impl VM {
        pub fn new(memory_size: usize) -> Self {
            VM {
                stack: Vec::with_capacity(memory_size),
            }
        }

        pub fn program_load(&mut self, _bin_file: &str) {
            // Load the program from the binary file
            // Implementation required
        }

        pub fn register_get(&self, _index: usize) -> u64 {
            // Get register value
            // Implementation required
            0
        }

        pub fn execute_program(&mut self) {
            // Execute the program
            // Implementation required
        }

        pub fn stack_pop(&mut self) -> u64 {
            self.stack.pop().unwrap_or(0)
        }
    }
}

use tiny_riscv64::VM;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <asm_file> <bin_file> [all|failed]", args[0]);
        process::exit(1);
    }

    let asm_file = &args[1];
    let bin_file = &args[2];
    let print_all = args.get(3).map_or(false, |arg| arg == "all");

    let mut stack_values: VecDeque<u64> = VecDeque::new();
    let mut vm = VM::new(4096);

    vm.program_load(bin_file);

    let sp_before = vm.register_get(2);

    if let Err(e) = panic::catch_unwind(|| vm.execute_program()) {
        eprintln!("VM Exception: {:?}", e);
        process::exit(1);
    }

    while vm.register_get(2) < sp_before {
        stack_values.push_front(vm.stack_pop());
    }

    let content = match fs::read_to_string(asm_file) {
        Ok(content) => content,
        Err(_) => {
            eprintln!("Failed to open asm file '{}'", asm_file);
            process::exit(1);
        }
    };

    let test_block_regex = Regex::new(r"(\# TEST:[\s\S]*?EXPECTED PUSH:\s*(0x[0-9A-Fa-f]+)[\s\S]*?)(?=\#\s*TEST|$)").unwrap();
    let mut test_cases = Vec::new();

    for cap in test_block_regex.captures_iter(&content) {
        if let Some(block) = cap.get(1) {
            if let Some(expected_str) = cap.get(2) {
                let expected_value = u64::from_str_radix(&expected_str.as_str()[2..], 16).unwrap();
                test_cases.push((block.as_str().to_string(), expected_value));
            }
        }
    }

    let mut passed = 0;
    let mut failed = 0;

    for (i, (block, expected)) in test_cases.iter().enumerate() {
        if let Some(actual) = stack_values.get(i) {
            let pass = *expected == *actual;

            if pass {
                passed += 1;
            } else {
                failed += 1;
            }

            if !pass || print_all {
                println!(
                    "{} Test {}:\nExpected: 0x{:016X}\nActual:   0x{:016X}\n{}",
                    if pass { "PASS" } else { "FAIL" },
                    i + 1,
                    expected,
                    actual,
                    block
                );
                println!();
            }
        }
    }

    println!("Passed: {}, Failed: {}", passed, failed);

    process::exit(failed);
}