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

// Assuming TinyRISCV64 module is available as an external dependency
// Uncomment the appropriate line or provide the file if needed
// mod tiny_riscv64; 
// use tiny_riscv64::VM;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <asm_file> <bin_file> [all|failed]", args[0]);
        std::process::exit(1);
    }

    let asm_file = &args[1];
    let bin_file = &args[2];
    let print_all = args.len() > 3 && args[3] == "all";

    let mut stack_values = VecDeque::new();

    // Placeholder for VM instantiation and error handling.
    // Replace the below line with actual implementation if/when available
    // let mut vm = VM::new(4096).expect("VM creation failed");

    // Sample code to mimic behavior; replace with actual logic
    // Replace the following lines with actual implementation if/when available
    let sp_before = 0; // Placeholder for register_get method

    // Error handling for executing program
    // if let Err(e) = vm.execute_program() {
    //     eprintln!("VM Exception: {}", e);
    //     std::process::exit(1);
    // }

    // Sample code for stack handling; replace with actual logic
    // Replace the following lines with actual implementation if/when available
    while false /* Replace with the condition for register_get less than sp_before */ {
        match Some(0) /* Placeholder for stack_pop method */ {
            Some(value) => stack_values.push_front(value),
            None => {
                eprintln!("Failed to pop from stack.");
                std::process::exit(1);
            }
        };
    }

    // Read and parse the ASM file
    let mut file = match File::open(asm_file) {
        Ok(file) => file,
        Err(_) => {
            eprintln!("Failed to open asm file '{}'", asm_file);
            std::process::exit(1);
        }
    };

    let mut content = String::new();
    if file.read_to_string(&mut content).is_err() {
        eprintln!("Failed to read asm file '{}'", asm_file);
        std::process::exit(1);
    }

    // Placeholder for regex logic (requires external crate)
    // Properties like test_block_regex and test_cases are substitutes
    let test_cases: Vec<(String, u64)> = Vec::new();

    let (mut passed, mut failed) = (0, 0);

    for (i, (block, expected)) in test_cases.iter().enumerate().take(stack_values.len()) {
        let actual = stack_values[i];
        let pass = expected == &actual;

        match pass {
            true => passed += 1,
            false => failed += 1
        }

        if !pass || print_all {
            println!("{} Test {}:", if pass { "PASS" } else { "FAIL" }, i + 1);
            println!("Expected: {:#018X}", expected);
            println!("Actual:   {:#018X}", actual);
            println!("{}", block);
            println!();
        }
    }

    println!("Passed: {}, Failed: {}", passed, failed);

    // Exit with the number of failed tests
    std::process::exit(failed);
}