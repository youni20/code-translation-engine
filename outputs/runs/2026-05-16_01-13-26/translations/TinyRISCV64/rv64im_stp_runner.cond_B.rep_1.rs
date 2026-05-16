use std::env;
use std::fs::File;
use std::io::Read;
use std::collections::VecDeque;
use std::process;
use std::error::Error;

// Assume TinyRISCV64 is a suitable Rust module or equivalent functionality is implemented.
mod tiny_riscv64 {
    pub struct VM {
        stack: Vec<u64>,
        sp: usize,
    }

    impl VM {
        pub fn new(stack_size: usize) -> VM {
            VM {
                stack: Vec::with_capacity(stack_size / 8),
                sp: stack_size,
            }
        }

        pub fn program_load(&mut self, _bin_file: &str) {
            // Simulated program load
        }

        pub fn execute_program(&mut self) {
            // Simulated program execution
        }

        pub fn register_get(&self, _index: usize) -> usize {
            self.sp
        }

        pub fn stack_pop(&mut self) -> u64 {
            self.sp += 8;
            self.stack.pop().unwrap_or(0)
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
    let print_all = args.get(3).map_or(false, |s| s == "all");

    let mut stack_values = VecDeque::<u64>::new();
    let mut vm = tiny_riscv64::VM::new(4096);

    if let Err(e) = (|| {
        vm.program_load(bin_file);
        let sp_before = vm.register_get(2);
        vm.execute_program();
        while vm.register_get(2) < sp_before {
            stack_values.push_front(vm.stack_pop());
        }
        Ok::<(), Box<dyn Error>>(())
    })() {
        eprintln!("VM Exception: {}", e);
        process::exit(1);
    }

    let mut file = File::open(asm_file)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    let test_blocks = parse_test_blocks(&content)?;

    let mut test_cases = Vec::new();
    for (block, expected_str) in test_blocks {
        let expected_value = u64::from_str_radix(&expected_str[2..], 16)?;
        test_cases.push((block, expected_value));
    }

    let mut passed = 0;
    let mut failed = 0;

    for (i, test_case) in test_cases.iter().enumerate().take(stack_values.len()) {
        let expected = test_case.1;
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
            println!("{}", test_case.0);
            println!();
        }
    }

    println!("Passed: {}, Failed: {}", passed, failed);
    process::exit(failed);
}

fn parse_test_blocks(content: &str) -> Result<Vec<(String, String)>, Box<dyn Error>> {
    let mut test_blocks = Vec::new();
    let mut lines = content.lines().peekable();

    while let Some(line) = lines.next() {
        if line.contains("# TEST:") {
            let mut block = String::from(line);
            while let Some(&next_line) = lines.peek() {
                if next_line.contains("# TEST:") {
                    break;
                }
                if next_line.contains("EXPECTED PUSH:") {
                    let parts: Vec<&str> = next_line.split_whitespace().collect();
                    if let Some(&expected) = parts.get(parts.len() - 1) {
                        test_blocks.push((block.clone(), String::from(expected)));
                        break;
                    }
                }
                block.push('\n');
                block.push_str(next_line);
                lines.next();
            }
        }
    }
    Ok(test_blocks)
}