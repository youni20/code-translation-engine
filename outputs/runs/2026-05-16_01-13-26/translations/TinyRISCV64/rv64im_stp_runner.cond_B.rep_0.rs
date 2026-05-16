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
use std::collections::VecDeque;
use std::fs::File;
use std::io::{self, Read};
use std::process;
use std::string::String;
use std::vec::Vec;

// Assume TinyRISCV64::VM and its methods are properly defined in a proper Rust module.
// Here is a mockup for illustrative purposes.
mod tiny_riscv64 {
    pub struct VM {
        // Implement or mock up the VM struct and methods.
    }

    impl VM {
        pub fn new(_stack_size: usize) -> Self {
            // Initialize a new VM with the given stack size
            VM {}
        }

        pub fn program_load(&mut self, _bin_file: &str) {
            // Load a program from the binary file
        }

        pub fn register_get(&self, _index: usize) -> u64 {
            // Get the value of a given register
            0
        }

        pub fn execute_program(&mut self) {
            // Execute the loaded program
        }

        pub fn stack_pop<T>(&mut self) -> T {
            // Pop a value from the VM stack
            unimplemented!()
        }
    }
}

use tiny_riscv64::VM;

fn main() -> io::Result<()> {
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

    vm.execute_program();

    // Dump the stack
    while vm.register_get(2) < sp_before {
        stack_values.push_front(vm.stack_pop::<u64>());
    }

    let mut file = File::open(asm_file)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    // Since we cannot use regex, we manually parse test cases
    let mut test_cases: Vec<(String, u64)> = Vec::new();
    for line in content.lines() {
        if line.contains("# TEST:") && line.contains("EXPECTED PUSH:") {
            if let Some(start) = line.find("0x") {
                if let Ok(expected_value) = u64::from_str_radix(&line[start + 2..].trim(), 16) {
                    test_cases.push((line.to_string(), expected_value));
                }
            }
        }
    }

    let mut passed = 0;
    let mut failed = 0;

    for (i, test_case) in test_cases.iter().enumerate() {
        if let Some(&actual) = stack_values.get(i) {
            let expected = test_case.1;
            let pass = expected == actual;

            if pass {
                passed += 1;
            } else {
                failed += 1;
            }

            if !pass || print_all {
                println!("{} Test {}:", if pass { "PASS" } else { "FAIL" }, i + 1);
                println!("Expected: 0x{:016X}", expected);
                println!("Actual:   0x{:016X}", actual);
                println!("{}", test_case.0);
                println!();
            }
        }
    }

    println!("Passed: {}, Failed: {}", passed, failed);

    process::exit(failed);
}