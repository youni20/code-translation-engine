use std::env;
use std::io::{self};
use std::sync::Arc;

mod tiny_elf_riscv64 {
    use std::sync::Arc;
    
    pub struct ElfVM;

    impl ElfVM {
        pub fn new() -> ElfVM {
            ElfVM
        }

        pub fn program_load(&self, _filename: &str) -> Result<u64, String> {
            // Simulated loading, return a dummy entry point for illustration.
            Ok(42)
        }

        pub fn map_fd(&self, fd: i32, _stream: Arc<dyn std::io::Read + Send + Sync>) {
            // Placeholder implementation
        }

        pub fn map_data_mem(&self, _data: *const u8, _size: usize) -> u64 {
            // Return a dummy virtual address
            1000
        }

        pub fn stack_push(&self, _value: u64) -> u64 {
            // Placeholder implementation
            2000
        }

        pub fn register_set(&self, _reg: i32, _value: u64) {
            // Placeholder implementation
        }

        pub fn execute_program(&self, _entry: u64, _limit: u64) {
            // Placeholder implementation
        }

        pub fn register_get(&self, _reg: i32) -> i32 {
            // Placeholder implementation
            0
        }
    }
}

fn main() {
    use tiny_elf_riscv64::ElfVM;

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <bin_file>", args[0]);
        std::process::exit(1);
    }
    let vm_bin_filename = &args[1];
    let vm = ElfVM::new();

    match vm.program_load(vm_bin_filename) {
        Ok(entry_point) => {
            vm.map_fd(0, Arc::new(io::stdin()) as Arc<dyn io::Read + Send + Sync>);

            let mut vm_arg_data: Vec<u8> = Vec::new();
            let mut vm_argv: Vec<u64> = Vec::new();

            for arg in args.iter().skip(1) {
                let bytes = arg.bytes().chain(std::iter::once(0)).collect::<Vec<_>>();
                let offset = vm_arg_data.len() as u64;
                vm_arg_data.extend_from_slice(&bytes);
                vm_argv.push(offset);
            }

            let virt_addr = vm.map_data_mem(vm_arg_data.as_ptr(), vm_arg_data.len());

            for av in &mut vm_argv {
                *av += virt_addr;
            }
            vm_argv.push(0);

            for av in vm_argv.iter().rev() {
                vm.stack_push(*av);
            }

            vm.register_set(10, (args.len() - 1) as u64);
            vm.register_set(11, virt_addr);

            vm.execute_program(entry_point, 100 * 1024 * 1024);
            std::process::exit(vm.register_get(10));
        }
        Err(e) => {
            eprintln!("VM Exception: {}", e);
            std::process::exit(1);
        }
    }
}