/*
 * MIT License
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
use std::io::{self, Read};
use std::process;

// Since the tiny_elf_riscv64 module is missing, let's define a bare minimum here
mod tiny_elf_riscv64 {
    use std::io::Read;

    pub struct ElfVM {}

    impl ElfVM {
        pub fn new(_size: usize) -> Self {
            ElfVM {}
        }

        pub fn program_load(&mut self, _bin_file: &str) -> Result<u64, &'static str> {
            Err("not implemented")
        }

        pub fn map_fd(&mut self, _fd: i32, _data: &mut dyn Read) {}

        pub fn execute_program(&mut self, _entry_point: u64, _steps: u64) {}

        pub fn as_mut_vm(&mut self) -> &mut VM {
            unimplemented!()
        }
    }

    pub struct VM {}

    impl VM {
        pub fn program_load(&mut self, _bin_file: &str) {}

        pub fn map_data_mem(&mut self, _data: &[u8]) -> u64 {
            0
        }

        pub fn stack_push(&mut self, _val: u64) -> u64 {
            0
        }

        pub fn register_set(&mut self, _index: usize, _value: u64) {}

        pub fn execute_program(&mut self) {}

        pub fn register_get(&self, _index: usize) -> u64 {
            0
        }

        pub fn stack_pop<T>(&mut self) -> T {
            unimplemented!()
        }
    }
}

use tiny_elf_riscv64::{ElfVM, VM};

fn run_raw(vm: &mut VM, bin_file: &str) -> i32 {
    match File::open(bin_file) {
        Ok(mut file) => {
            let mut buf = vec![0u8; 1024];
            file.read_exact(&mut buf).unwrap();
            
            let mut x: u64 = 0x0123456789abcdef;
            for i in 0..1024 {
                x = x.wrapping_mul(6364136223846793005).wrapping_add(1);
                buf[i] = (x >> 56) as u8;
            }
            let native_buf = buf.clone();

            vm.program_load(bin_file);
            let data_addr_buf = vm.map_data_mem(&buf);

            let stack_addr_src = vm.stack_push(0u64);
            let stack_addr_dst = vm.stack_push(0u64);

            vm.register_set(10, data_addr_buf);
            vm.register_set(11, buf.len() as u64);
            vm.register_set(12, stack_addr_src);
            vm.register_set(13, stack_addr_dst);

            vm.execute_program();

            let res = vm.register_get(10);
            let dst = vm.stack_pop::<u64>();
            let src = vm.stack_pop::<u64>();
            println!("res = 0x{:016x}", res);
            println!("src = 0x{:016x}", src);
            println!("dst = 0x{:016x}", dst);

            let (native_res, native_src, native_dst) = get_addrs(&native_buf);
            println!("native_res = 0x{:016x}", native_res);
            println!("native_src = 0x{:016x}", native_src);
            println!("native_dst = 0x{:016x}", native_dst);

            println!("Buffer equal : {}", native_buf == buf);
            println!("Result equal : {}", native_res == res);
            println!("Source equal : {}", native_src == src);
            println!("Destin equal : {}", native_dst == dst);

            if native_buf != buf || native_res != res || native_src != src || native_dst != dst {
                return 1;
            }
        }
        Err(e) => {
            eprintln!("error: {}", e);
            return 1;
        }
    }
    0
}

fn run_elf(vm: &mut ElfVM, data_file: &str, entry_point: u64) -> i32 {
    match File::open(data_file) {
        Ok(mut pDataStream) => {
            let mut pOutStream: Vec<u8> = vec![];
            let mut pErrStream: Vec<u8> = vec![];

            vm.map_fd(0, &mut pDataStream);
            vm.map_fd(1, &mut io::Cursor::new(&mut pOutStream));
            vm.map_fd(2, &mut io::Cursor::new(&mut pErrStream));

            vm.execute_program(entry_point, 100_000_000);

            let vm_output = String::from_utf8_lossy(&pOutStream).into_owned();
            let mut data_stream_clone = pDataStream.try_clone().unwrap();
            let native_output = compute_sha512(&mut data_stream_clone);

            if vm_output != native_output {
                eprintln!(
                    "Program output: '{}' != '{}'\nProgram StdErr: '{}'", 
                    vm_output, 
                    native_output, 
                    String::from_utf8_lossy(&pErrStream)
                );
                return 1;
            }
        }
        Err(e) => {
            eprintln!("error: {}", e);
            return 1;
        }
    }
    println!("PASS");
    0
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <bin_file> [data_file]", args[0]);
        process::exit(1);
    }
    let bin_file = &args[1];

    let mut vm = ElfVM::new(4096);
    let bin_is_elf;
    let data_file;
    let entry_point;
    match vm.program_load(bin_file) {
        Ok(ep) => {
            entry_point = ep;
            bin_is_elf = true;
            if args.len() < 3 {
                eprintln!("Error: no data file provided.");
                process::exit(1);
            }
            data_file = args[2].clone();
        }
        Err(e) => {
            eprintln!("Loading as elf failed: '{}' , assuming raw bytecode.", e);
            bin_is_elf = false;
            data_file = String::new();
            entry_point = 0;
        }
    }

    let exit_code = if bin_is_elf {
        run_elf(&mut vm, &data_file, entry_point)
    } else {
        run_raw(vm.as_mut_vm(), bin_file)
    };

    process::exit(exit_code);
}

// Placeholder function for native logic
fn get_addrs(_data: &[u8]) -> (u64, u64, u64) {
    (0, 0, 0) // Replace with actual implementation
}

// Placeholder function for SHA-512 computation
fn compute_sha512<R: Read>(_reader: &mut R) -> String {
    "fake_sha512".to_string() // Replace with actual implementation
}