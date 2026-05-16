use std::env;
use std::io::{self};
use std::process;
use std::sync::Arc;

// Since `tiny_elf_riscv64` is assumed to be a Rust module, 
// let's define it here for the purpose of having a complete, compile-ready code.
// In actual practice, this would be an external module or crate.
mod tiny_elf_riscv64 {
    use std::sync::Arc;

    pub struct ElfVM;

    impl ElfVM {
        pub fn new() -> Self {
            ElfVM
        }

        pub fn program_load(&mut self, _filename: &str) -> Result<u64, Box<dyn std::error::Error>> {
            Ok(0) // Mock implementation
        }

        pub fn map_fd(&mut self, _fd: i32, _handle: Arc<dyn std::io::Read + Send + Sync>) {}

        pub fn map_data_mem(&mut self, _data: &[u8]) -> Result<u64, Box<dyn std::error::Error>> {
            Ok(0) // Mock implementation
        }

        pub fn stack_push(&mut self, _value: u64) -> Result<u64, Box<dyn std::error::Error>> {
            Ok(0) // Mock implementation
        }

        pub fn register_set(&mut self, _reg: u8, _value: u64) {}

        pub fn register_get(&self, _reg: u8) -> i32 {
            0 // Mock implementation
        }

        pub fn execute_program(&mut self, _entry_point: u64, _memory_limit: u64) -> Result<(), Box<dyn std::error::Error>> {
            Ok(()) // Mock implementation
        }
    }
}

use tiny_elf_riscv64::ElfVM; // Removed unused imports

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <bin_file>", args[0]);
        process::exit(1);
    }

    let vm_bin_filename = &args[1];
    if let Err(e) = execute_vm(vm_bin_filename, &args) {
        eprintln!("VM Exception: {}", e);
        process::exit(1);
    }
}

fn execute_vm(vm_bin_filename: &str, args: &[String]) -> Result<i32, Box<dyn std::error::Error>> {
    let mut vm = ElfVM::new(); // Assuming a constructor method `new`

    let entry_point = vm.program_load(vm_bin_filename)?;

    let stdin = Arc::new(io::stdin());

    // Using standard FDs values
    vm.map_fd(0, stdin as Arc<dyn std::io::Read + Send + Sync>); // STDIN_FILENO

    let mut vm_arg_data = Vec::new();
    let mut vm_argv = Vec::new();

    for arg in &args[1..] {
        let n = arg.len() + 1;
        let offset = vm_arg_data.len();
        vm_arg_data.extend_from_slice(arg.as_bytes());
        vm_arg_data.push(0); // Add null terminator
        vm_argv.push(offset as u64);
    }

    let virt_addr = vm.map_data_mem(&vm_arg_data)?;

    for av in &mut vm_argv {
        *av += virt_addr;
    }
    vm_argv.push(0);

    for &arg in vm_argv.iter().rev() {
        let new_addr = vm.stack_push(arg)?;
        vm.register_set(11, new_addr); // a1 = argv
    }

    vm.register_set(10, (args.len() - 1) as u64); // a0 = argc

    vm.execute_program(entry_point, 100 * 1024 * 1024)?;

    Ok(vm.register_get(10))
}