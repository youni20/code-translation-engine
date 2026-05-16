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
use std::io::{self, BufRead};
use std::collections::VecDeque;
use std::process;

mod tiny_riscv64 {
    pub struct VM {
        stack: Vec<u64>,
        sp: usize,
    }

    impl VM {
        pub fn new(stack_size: usize) -> Self {
            Self {
                stack: vec![0; stack_size],
                sp: stack_size,
            }
        }

        pub fn program_load(&mut self, _filename: &str) {
            // Simulating binary program load
        }

        pub fn register_get(&self, reg: usize) -> usize {
            if reg == 2 {
                self.sp
            } else {
                0
            }
        }

        pub fn execute_program(&mut self) {
            // Simulate executing some program which modifies the stack
            self.sp -= 1;
            self.stack[self.sp] = 0xDEADBEEFCAFEBABE;
            self.sp -= 1;
            self.stack[self.sp] = 0xBAADF00DDEADFACE;
        }

        pub fn stack_pop<T: Copy>(&mut self) -> T {
            self.sp += std::mem::size_of::<T>();
            unsafe { *(&self.stack[self.sp] as *const _ as *const T) }
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
    let print_all = args.len() > 3 && args[3] == "all";

    let mut stack_values = VecDeque::new();
    let mut vm = tiny_riscv64::VM::new(4096);
    vm.program_load(bin_file);

    let sp_before = vm.register_get(2);
    vm.execute_program();

    while vm.register_get(2) < sp_before {
        stack_values.push_front(vm.stack_pop::<u64>());
    }

    let file = File::open(asm_file);
    if file.is_err() {
        eprintln!("Failed to open asm file '{}'", asm_file);
        process::exit(1);
    }
    let content = io::BufReader::new(file.unwrap())
        .lines()
        .map(|l| l.unwrap())
        .collect::<Vec<String>>()
        .join("\n");

    let mut test_cases: Vec<(String, u64)> = Vec::new();

    for block in content.split("# TEST").skip(1) {
        if let Some(pos) = block.find("EXPECTED PUSH:") {
            if let Some(start) = block[pos..].find("0x") {
                let value_str = block[pos + start..].split_whitespace().next().unwrap();
                if let Ok(expected_value) = u64::from_str_radix(&value_str[2..], 16) {
                    test_cases.push((block.to_string(), expected_value));
                }
            }
        }
    }

    let mut passed = 0;
    let mut failed = 0;

    for (i, case) in test_cases.iter().enumerate().take(stack_values.len()) {
        let expected = case.1;
        let actual = stack_values[i];
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
            println!("{}", case.0);
            println!();
        }
    }

    println!("Passed: {}, Failed: {}", passed, failed);
    process::exit(failed);
}