use std::env;
use std::io::{self, Read};
use std::process;
use std::sync::Arc;
use TinyElfRISCV64::ElfVM;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <bin_file>", args[0]);
        process::exit(1);
    }
    let vm_bin_filename = &args[1];

    let result = (|| -> Result<i32, Box<dyn std::error::Error>> {
        let mut vm = ElfVM::new();

        let entry_point = vm.program_load(vm_bin_filename)?;

        // For stdout and stderr, we need a Read wrapper
        let stdout_read: Arc<dyn Read + Send + Sync> = Arc::new(io::stdin());
        let stderr_read: Arc<dyn Read + Send + Sync> = Arc::new(io::stdin());

        vm.map_fd(0, Arc::new(io::stdin().lock()));
        vm.map_fd(1, stdout_read);
        vm.map_fd(2, stderr_read);

        let mut vm_arg_data = Vec::new();
        let mut vm_argv = Vec::new();
        for arg in args.iter().skip(1) {
            let bytes = arg.as_bytes();
            let offset = vm_arg_data.len();
            vm_arg_data.extend_from_slice(bytes);
            vm_arg_data.push(0);
            vm_argv.push(offset as u64);
        }

        let virt_addr = vm.map_data_mem(&vm_arg_data)?;

        for av in vm_argv.iter_mut() {
            *av += virt_addr;
        }
        vm_argv.push(0);

        for &arg_addr in vm_argv.iter().rev() {
            vm.stack_push(arg_addr)?;
        }

        vm.register_set(10, (args.len() - 1) as u64); 
        vm.register_set(11, virt_addr);

        vm.execute_program(entry_point, 100 * 1024 * 1024)?;

        Ok(vm.register_get(10) as i32)
    })();

    match result {
        Ok(code) => process::exit(code),
        Err(e) => {
            eprintln!("VM Exception: {}", e);
            process::exit(1);
        }
    }
}

mod TinyElfRISCV64 {
    use std::sync::Arc;
    use std::io::Read;

    pub struct ElfVM;

    impl ElfVM {
        pub fn new() -> Self {
            ElfVM
        }

        pub fn program_load(&mut self, _filename: &str) -> Result<u64, Box<dyn std::error::Error>> {
            Ok(0)
        }

        pub fn map_fd<R: Read + ?Sized>(&mut self, _fd: i32, _stream: Arc<R>) {}

        pub fn map_data_mem(&mut self, _data: &[u8]) -> Result<u64, Box<dyn std::error::Error>> {
            Ok(0)
        }

        pub fn stack_push(&mut self, value: u64) -> Result<u64, Box<dyn std::error::Error>> {
            Ok(value)
        }

        pub fn register_set(&mut self, _register: i32, _value: u64) {}

        pub fn execute_program(&self, _entry_point: u64, _limit: u64) -> Result<(), Box<dyn std::error::Error>> {
            Ok(())
        }

        pub fn register_get(&self, _register: i32) -> u64 {
            0
        }
    }
}