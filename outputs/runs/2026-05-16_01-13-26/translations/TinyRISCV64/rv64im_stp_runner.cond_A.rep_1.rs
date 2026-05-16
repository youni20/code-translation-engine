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

use crate::tiny_riscv64::VM;

mod tiny_riscv64 {
    pub struct VM {
        // Placeholder for actual implementation
    }

    impl VM {
        pub fn new(_stack_size: usize) -> Self {
            VM {}
        }

        pub fn program_load(&mut self, _bin_file: &str) {
            // Placeholder for actual implementation
        }

        pub fn register_get(&self, _register: usize) -> u64 {
            // Placeholder for actual implementation
            0
        }

        pub fn execute_program(&mut self) {
            // Placeholder for actual implementation
        }

        pub fn stack_pop<T>(&mut self) -> T {
            // Placeholder for actual implementation
            unimplemented!()
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <asm_file> <bin_file> [all|failed]", args[0]);
        process::exit(1);
    }

    let asm_file = &args[1];
    let bin_file = &args[2];
    let print_all = args.get(3).map_or(false, |arg| arg == "all");

    let mut stack_values = VecDeque::new();

    let mut vm = VM::new(4096);
    vm.program_load(bin_file);

    let sp_before = vm.register_get(2);

    vm.execute_program();

    while vm.register_get(2) < sp_before {
        stack_values.push_front(vm.stack_pop::<u64>());
    }

    let content = fs::read_to_string(asm_file).unwrap_or_else(|_| {
        eprintln!("Failed to open asm file '{}'", asm_file);
        process::exit(1);
    });

    // Use of regex is removed since it's an external crate and cannot be used based on the constraints.
    // Simulating a simple processing without regex since it's removed.
    let test_cases = parse_test_cases(&content);

    let mut passed = 0;
    let mut failed = 0;

    for (i, (expected, actual)) in test_cases.iter().zip(stack_values.iter()).enumerate() {
        let pass = expected.1 == *actual;

        if pass {
            passed += 1;
        } else {
            failed += 1;
        }

        if !pass || print_all {
            println!("{} Test {}:", if pass { "PASS" } else { "FAIL" }, i + 1);
            println!("Expected: 0x{:016X}", expected.1);
            println!("Actual:   0x{:016X}", actual);
            println!("{}", expected.0);
            println!();
        }
    }

    println!("Passed: {}, Failed: {}", passed, failed);

    process::exit(failed);
}

// A mock of the parsing function since we're not using regex
fn parse_test_cases(content: &str) -> Vec<(String, u64)> {
    let mut test_cases = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        if lines[i].contains("# TEST:") {
            let block_start = i;
            let mut expected_value: Option<u64> = None;

            while i < lines.len() && !lines[i].contains("# TEST:") {
                if let Some(pos) = lines[i].find("EXPECTED PUSH:") {
                    let hex_str = lines[i][pos + 15..].trim();
                    expected_value = u64::from_str_radix(&hex_str[2..], 16).ok();
                    break;
                }
                i += 1;
            }

            if let Some(val) = expected_value {
                let block = lines[block_start..=i].join("\n");
                test_cases.push((block, val));
            }
        }
        i += 1;
    }

    test_cases
}