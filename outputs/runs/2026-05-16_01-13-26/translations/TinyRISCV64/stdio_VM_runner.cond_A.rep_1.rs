use std::env;
use std::io::{self};
use std::process;

mod tiny_elf_riscv64; // Ensure the file exists: 'tiny_elf_riscv64.rs' or 'tiny_elf_riscv64/mod.rs'

use tiny_elf_riscv64::ElfVM;
use tiny_elf_riscv64::{TinyU8, TinyU64};

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <bin_file>", args[0]);
        process::exit(1);
    }
    
    let vm_bin_filename = &args[1];
    match run_vm(vm_bin_filename, &args) {
        Ok(exit_code) => process::exit(exit_code),
        Err(e) => {
            eprintln!("VM Exception: {}", e);
            process::exit(1);
        }
    }
}

fn run_vm(vm_bin_filename: &str, args: &[String]) -> Result<i32, Box<dyn std::error::Error>> {
    let mut vm = ElfVM::new();
    let entry_point = vm.program_load(vm_bin_filename)?;
    
    vm.map_fd(0, Box::new(io::stdin()));
    vm.map_fd(1, Box::new(io::stdout()));
    vm.map_fd(2, Box::new(io::stderr()));
    
    let mut vm_arg_data: Vec<TinyU8> = Vec::new();
    let mut vm_argv: Vec<TinyU64> = Vec::new();
    
    for arg in args.iter().skip(1) {
        let n = arg.len() + 1;
        let offset = vm_arg_data.len();
        vm_arg_data.extend_from_slice(arg.as_bytes());
        vm_arg_data.push(0); // Add null terminator
        vm_argv.push(offset as TinyU64);
    }
    
    let mut virt_addr = vm.map_data_mem(&vm_arg_data)?;
    
    for av in vm_argv.iter_mut() {
        *av += virt_addr as TinyU64;
    }
    vm_argv.push(0);
    
    for &arg in vm_argv.iter().rev() {
        virt_addr = vm.stack_push(arg)?;
    }
    
    vm.register_set(10, (args.len() - 1) as TinyU64);
    vm.register_set(11, virt_addr as TinyU64);
    
    vm.execute_program(entry_point, 100 * 1024 * 1024)?;
    
    Ok(vm.register_get(10) as i32)
}