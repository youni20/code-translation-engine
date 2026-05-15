use std::env;
use std::io::{self};
use std::process;
use std::sync::{Arc, Mutex};

mod tiny_elf_riscv64 {
    use std::io;
    use std::sync::{Arc, Mutex};

    pub struct ElfVM;

    impl ElfVM {
        pub fn new() -> Self {
            ElfVM
        }

        pub fn program_load(&mut self, _filename: &str) -> Result<u64, &'static str> {
            // Dummy implementation
            Ok(0)
        }

        pub fn map_fd(&mut self, _fd: i32, _stream: Arc<Mutex<dyn io::Read + Send>>) {
            // Dummy implementation
        }

        pub fn map_data_mem(&mut self, data: &[u8]) -> u64 {
            // Dummy implementation
            data.len() as u64
        }

        pub fn stack_push(&mut self, value: u64) -> u64 {
            // Dummy implementation
            value
        }

        pub fn register_set(&mut self, _reg: u8, _value: u64) {
            // Dummy implementation
        }

        pub fn execute_program(&mut self, _entry_point: u64, _cycles: u64) {
            // Dummy implementation
        }

        pub fn register_get(&self, _reg: u8) -> i32 {
            // Dummy implementation
            0
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
    let mut vm = tiny_elf_riscv64::ElfVM::new();

    if let Ok(entry_point) = vm.program_load(vm_bin_filename) {
        vm.map_fd(0, Arc::new(Mutex::new(io::stdin())));
        // Prevent mapping stdout and stderr as they are not Read
        // vm.map_fd(1, Arc::new(Mutex::new(io::stdout())));
        // vm.map_fd(2, Arc::new(Mutex::new(io::stderr())));

        let mut vm_arg_data = Vec::new();
        let mut vm_argv = Vec::new();
        for arg in &args[1..] {
            let bytes = arg.as_bytes();
            vm_arg_data.extend_from_slice(bytes);
            vm_arg_data.push(0); // null-terminator
            vm_argv.push(vm_arg_data.len() as u64 - bytes.len() as u64 - 1);
        }

        let virt_addr = vm.map_data_mem(&vm_arg_data);

        for av in vm_argv.iter_mut() {
            *av += virt_addr;
        }
        vm_argv.push(0);

        let mut virt_addr = virt_addr;
        for &arg_ptr in vm_argv.iter().rev() {
            virt_addr = vm.stack_push(arg_ptr);
        }

        vm.register_set(10, (args.len() - 1) as u64); // a0 = argc
        vm.register_set(11, virt_addr);               // a1 = argv

        vm.execute_program(entry_point, 100 * 1024 * 1024);

        process::exit(vm.register_get(10));
    } else {
        eprintln!("Error loading program");
        process::exit(1);
    }
}