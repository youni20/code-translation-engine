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
use std::fs::File;
use std::io::Read;
use std::collections::VecDeque;
use std::process::exit;

mod tinyriscv64 {
    pub struct VM {
        // Assuming a placeholder for the actual implementation details
        stack: Vec<u64>,
        registers: [u64; 32],
    }

    impl VM {
        pub fn new(stack_size: usize) -> Self {
            VM {
                stack: Vec::with_capacity(stack_size / 8),
                registers: [0; 32],
            }
        }

        pub fn program_load(&mut self, _bin_file: &str) -> Result<(), String> {
            // Load binary into VM (simulated)
            Ok(())
        }

        pub fn register_get(&self, index: usize) -> u64 {
            self.registers[index]
        }

        pub fn execute_program(&mut self) -> Result<(), String> {
            // Execute loaded program (simulated)
            Ok(())
        }

        pub fn stack_pop(&mut self) -> u64 {
            self.stack.pop().unwrap()
        }
    }
}

use crate::tinyriscv64::VM;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <asm_file> <bin_file> [all|failed]", args[0]);
        exit(1);
    }

    let asm_file = &args[1];
    let bin_file = &args[2];
    let print_all = args.len() > 3 && args[3] == "all";

    let mut stack_values = VecDeque::new();

    let mut vm = VM::new(4096);
    let load_result = vm.program_load(bin_file);
    if let Err(e) = load_result {
        eprintln!("Error loading program: {}", e);
        exit(1);
    }

    let sp_before = vm.register_get(2);

    let exec_result = vm.execute_program();
    if let Err(e) = exec_result {
        eprintln!("VM Exception: {}", e);
        exit(1);
    }

    while vm.register_get(2) < sp_before {
        stack_values.push_front(vm.stack_pop());
    }

    let mut file = match File::open(asm_file) {
        Ok(f) => f,
        Err(_) => {
            eprintln!("Failed to open asm file '{}'", asm_file);
            exit(1);
        }
    };

    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();

    let test_block_pattern = r"(?m)(# TEST:[\s\S]*?EXPECTED PUSH:\s*(0x[0-9A-Fa-f]+)[\s\S]*?)(?=# TEST|$)";
    let mut test_cases = Vec::new();

    for cap in content.lines().collect::<Vec<&str>>().windows(2) {
        if let Some(block) = cap.get(0) {
            if let Some(expected_line) = cap.get(1) {
                if expected_line.contains("EXPECTED PUSH:") {
                    let expected_str = expected_line.trim().split_whitespace().last().unwrap();
                    if expected_str.starts_with("0x") {
                        if let Ok(expected_value) = u64::from_str_radix(&expected_str[2..], 16) {
                            test_cases.push((block.to_string(), expected_value));
                        }
                    }
                }
            }
        }
    }

    let mut passed = 0;
    let mut failed = 0;

    for (i, (block, expected)) in test_cases.iter().enumerate().take(stack_values.len()) {
        let actual = stack_values[i];
        let pass = expected == &actual;

        if pass {
            passed += 1;
        } else {
            failed += 1;
        }

        if !pass || print_all {
            println!("{} Test {}:", if pass { "PASS" } else { "FAIL" }, i + 1);
            println!("Expected: 0x{:016X}", expected);
            println!("Actual:   0x{:016X}", actual);
            println!("{}", block);
            println!();
        }
    }

    println!("Passed: {}, Failed: {}", passed, failed);

    exit(failed);
}