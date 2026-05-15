use std::env;
use std::ffi::CString;
use std::io::{self};
use std::os::unix::io::AsRawFd;
use std::process;
use std::sync::Arc;

mod tinyriscv64 {
    pub type U8 = u8;
    pub type U64 = u64;

    pub struct ElfVM;

    impl ElfVM {
        pub fn new() -> Self {
            ElfVM
        }

        pub fn program_load(&mut self, _filename: String) -> usize {
            // Load the binary file and return the entry point
            unimplemented!()
        }

        pub fn map_fd<R: std::io::Read + std::io::Write + 'static>(&mut self, _fd: i32, _io: std::sync::Arc<R>) {
            // Map file descriptor to VM
            unimplemented!()
        }

        pub fn map_data_mem(&mut self, _data: &[U8], _size: usize) -> U64 {
            // Map data to VM memory
            unimplemented!()
        }

        pub fn stack_push(&mut self, _value: U64) -> U64 {
            // Push value onto VM stack
            unimplemented!()
        }

        pub fn register_set(&mut self, _index: i32, _value: U64) {
            // Set VM register
            unimplemented!()
        }

        pub fn register_get(&self, _index: i32) -> i32 {
            // Get VM register value
            unimplemented!()
        }

        pub fn execute_program(&mut self, _entry_point: usize, _memory_limit: u64) {
            // Execute program
            unimplemented!()
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <bin_file>", args[0]);
        process::exit(1);
    }
    let vm_bin_filename = &args[1];
    match run_vm(vm_bin_filename) {
        Ok(exit_code) => process::exit(exit_code),
        Err(e) => {
            eprintln!("VM Exception: {}", e);
            process::exit(1);
        }
    }
}

fn run_vm(vm_bin_filename: &str) -> Result<i32, Box<dyn std::error::Error>> {
    let mut vm = tinyriscv64::ElfVM::new();
    let entry_point = vm.program_load(vm_bin_filename.to_string());

    vm.map_fd(io::stdin().as_raw_fd(), Arc::new(io::stdin()));
    vm.map_fd(io::stdout().as_raw_fd(), Arc::new(io::stdout()));
    vm.map_fd(io::stderr().as_raw_fd(), Arc::new(io::stderr()));

    let mut vm_arg_data: Vec<tinyriscv64::U8> = Vec::new();
    let mut vm_argv: Vec<tinyriscv64::U64> = Vec::new();
    for arg in env::args().skip(1) {
        let cstr = CString::new(arg)?;
        let bytes = cstr.to_bytes_with_nul();
        let offset = vm_arg_data.len();
        vm_arg_data.extend_from_slice(bytes);
        vm_argv.push(offset as tinyriscv64::U64);
    }
    let virt_addr = vm.map_data_mem(&vm_arg_data, vm_arg_data.len());

    for av in &mut vm_argv {
        *av += virt_addr;
    }
    vm_argv.push(0);

    for &av in vm_argv.iter().rev() {
        vm.stack_push(av);
    }

    vm.register_set(10, (vm_argv.len() - 1) as tinyriscv64::U64);
    vm.register_set(11, virt_addr);

    vm.execute_program(entry_point, 100 * 1024 * 1024);
    Ok(vm.register_get(10))
}