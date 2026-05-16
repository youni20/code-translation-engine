use std::env;
use std::fs::File;
use std::os::unix::io::FromRawFd;
use std::process;

// Assume the module `tiny_elf_riscv64` is correctly defined elsewhere.
mod tiny_elf_riscv64 {
    pub struct ElfVM;

    impl ElfVM {
        pub fn new() -> Self {
            // Placeholder for actual implementation
            ElfVM
        }

        pub fn program_load(&mut self, _filename: &str) -> Result<u64, String> {
            // Placeholder for actual implementation
            Ok(0) // Dummy entry point
        }

        pub fn map_fd(&mut self, _fd: i32, _stream: Box<dyn crate::IoBoxed>) {
            // Placeholder for actual implementation
        }

        pub fn map_data_mem(&mut self, _data: &[u8]) -> u64 {
            // Placeholder for actual implementation
            0
        }

        pub fn stack_push(&mut self, _value: u64) -> u64 {
            // Placeholder for actual implementation
            0
        }

        pub fn register_set(&mut self, _reg: usize, _value: u64) {
            // Placeholder for actual implementation
        }

        pub fn execute_program(&mut self, _entry_point: u64, _memory_limit: u64) {
            // Placeholder for actual implementation
        }

        pub fn register_get(&self, _reg: usize) -> i32 {
            // Placeholder for actual implementation
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

    match vm.program_load(vm_bin_filename) {
        Ok(entry_point) => {
            vm.map_fd(0, Box::new(unsafe { File::from_raw_fd(0).into_io_boxed() }));
            vm.map_fd(1, Box::new(unsafe { File::from_raw_fd(1).into_io_boxed() }));
            vm.map_fd(2, Box::new(unsafe { File::from_raw_fd(2).into_io_boxed() }));

            let mut vm_arg_data = Vec::new();
            let mut vm_argv = Vec::new();
            for arg in &args[1..] {
                let arg_bytes = arg.as_bytes();
                vm_arg_data.extend_from_slice(arg_bytes);
                vm_arg_data.push(0);
                vm_argv.push(vm_arg_data.len() - arg_bytes.len() - 1);
            }

            let virt_addr = vm.map_data_mem(&vm_arg_data);

            for av in &mut vm_argv {
                *av += virt_addr as usize;
            }
            vm_argv.push(0);

            for &arg in vm_argv.iter().rev() {
                vm.stack_push(arg as u64);
            }

            vm.register_set(10, (args.len() - 1) as u64); // a0 = argc
            vm.register_set(11, virt_addr);               // a1 = argv

            vm.execute_program(entry_point, 100 * 1024 * 1024);

            process::exit(vm.register_get(10) as i32);
        }
        Err(e) => {
            eprintln!("VM Exception: {}", e);
            process::exit(1);
        }
    }
}

// Define a new trait that combines Read and Write
pub trait IoBoxed: std::io::Read + std::io::Write {}

impl<T: std::io::Read + std::io::Write> IoBoxed for T {}

// Extend the capability of File to be transformed into a Box<dyn IoBoxed>
trait IntoIOBoxed {
    fn into_io_boxed(self) -> Box<dyn IoBoxed>;
}

impl IntoIOBoxed for File {
    fn into_io_boxed(self) -> Box<dyn IoBoxed> {
        Box::new(self)
    }
}