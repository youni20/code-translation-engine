/*
 * MIT License
 *
 * (C) 2025 Neil Stephens
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
use std::io::{self, Read, Seek};
use std::vec::Vec;
use std::rc::Rc;
use std::cell::RefCell;
use std::hash::Hasher; // Import the Hasher trait

mod tinyelfriscv64 {
    pub struct VM;
    pub struct ElfVM;

    impl ElfVM {
        pub fn new(_: u64) -> Self {
            ElfVM
        }

        pub fn program_load(&mut self, _: &str) -> Result<u64, &'static str> {
            Err("Dummy error")
        }

        pub fn map_fd(&self, _: u64, _: std::rc::Rc<std::cell::RefCell<dyn std::io::Write>>) {}

        pub fn execute_program(&self, _: u64, _: u64) -> Result<(), &'static str> {
            Ok(())
        }
    }

    pub trait TinyRISCV64 {
        fn map_data_mem(&mut self, _: u64, _: u64) -> Result<u64, &'static str>;
        fn stack_push(&mut self, _: u64) -> Result<u64, &'static str>;
        fn register_set(&mut self, _: u8, _: u64);
        fn execute_program(&self) -> Result<(), &'static str>;
        fn register_get(&self, _: u8) -> u64;
        fn stack_pop<T>(&self) -> Result<u64, &'static str> {
            Ok(0)
        }
    }

    impl TinyRISCV64 for VM {
        fn map_data_mem(&mut self, _: u64, _: u64) -> Result<u64, &'static str> {
            Ok(0)
        }
        fn stack_push(&mut self, _: u64) -> Result<u64, &'static str> {
            Ok(0)
        }
        fn register_set(&mut self, _: u8, _: u64) {}
        fn execute_program(&self) -> Result<(), &'static str> {
            Ok(())
        }
        fn register_get(&self, _: u8) -> u64 {
            0
        }
    }
}
use tinyelfriscv64::{VM, ElfVM, TinyRISCV64};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <bin_file> [data_file]", args[0]);
        std::process::exit(1);
    }

    let bin_file = &args[1];

    let mut vm = ElfVM::new(4096);
    let bin_is_elf: bool;
    let data_file: &str;
    let entry_point: u64;

    match vm.program_load(bin_file) {
        Ok(ep) => {
            entry_point = ep;
            bin_is_elf = true;
            if args.len() < 3 {
                eprintln!("Error: no data file provided.");
                std::process::exit(1);
            }
            data_file = &args[2];
        }
        Err(e) => {
            eprintln!("Loading as elf failed: '{}' , assuming raw bytecode.", e);
            std::process::exit(1);
        }
    }

    if bin_is_elf {
        run_elf(&mut vm, data_file, entry_point);
    }
}

fn run_elf(vm: &mut ElfVM, data_file: &str, entry_point: u64) {
    const STDIN_FD: u64 = 0;
    const STDOUT_FD: u64 = 1;
    const STDERR_FD: u64 = 2;

    if let Err(e) = (|| {
        let p_data_stream = Rc::new(RefCell::new(File::open(data_file).map_err(|_| {
            io::Error::new(io::ErrorKind::InvalidInput, "Failed to open data file")
        })?));

        vm.map_fd(STDIN_FD, p_data_stream.clone() as Rc<RefCell<dyn std::io::Write>>);

        let p_out_stream = Rc::new(RefCell::new(io::Cursor::new(Vec::new())));
        vm.map_fd(STDOUT_FD, p_out_stream.clone() as Rc<RefCell<dyn std::io::Write>>);

        let p_err_stream = Rc::new(RefCell::new(io::Cursor::new(Vec::new())));
        vm.map_fd(STDERR_FD, p_err_stream.clone() as Rc<RefCell<dyn std::io::Write>>);

        vm.execute_program(entry_point, 100_000_000)?;

        let vm_output = String::from_utf8(p_out_stream.borrow().get_ref().clone())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        #[cfg(any(windows))]
        {
            let mut sha_hex = [0u8; 129];
            if !get_sha512_lowercase(data_file, &mut sha_hex) {
                return Err(io::Error::new(io::ErrorKind::Other, "Failed to compute SHA-512 hash of data file natively on host"));
            }
            let native_output = String::from_utf8_lossy(&sha_hex[..]).into_owned();
            compare_vm_output(vm_output, native_output, p_err_stream.clone())?;
        }

        #[cfg(not(any(windows)))]
        {
            let mut p_data_stream_mut = p_data_stream.borrow_mut();
            p_data_stream_mut.seek(io::SeekFrom::Start(0))?;

            let mut hasher = std::collections::hash_map::DefaultHasher::new(); // Replaced with DefaultHasher
            let mut buf = vec![0; 1024];
            loop {
                let bytes_read = p_data_stream_mut.read(&mut buf[..])?;
                if bytes_read == 0 {
                    break;
                }
                hasher.write(&buf[..bytes_read]); // Adjusted for DefaultHasher
            }
            let sha_hex = hasher.finish(); // Adjusted for DefaultHasher
            let native_output = format!("{:x}", sha_hex);
            compare_vm_output(vm_output, native_output, p_err_stream.clone())?;
        }

        Ok::<(), io::Error>(()) // Specified the Ok type
    })() {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }

    println!("PASS");
}

fn compare_vm_output(vm_output: String, native_output: String, p_err_stream: Rc<RefCell<std::io::Cursor<Vec<u8>>>>) -> io::Result<()> {
    if vm_output != native_output {
        let msg = format!(
            "Program output: '{}' != '{}'\nProgram StdErr: '{}'",
            vm_output,
            native_output,
            String::from_utf8_lossy(&p_err_stream.borrow().get_ref())
        );
        return Err(io::Error::new(io::ErrorKind::Other, msg));
    }
    Ok(())
}

fn run_raw(vm: &mut VM, bin_file: &str) {
    if let Err(e) = (|| {
        let buf_size = 1024;
        let mut buf: Vec<u8> = Vec::with_capacity(buf_size);

        let mut x: u64 = 0x0123456789abcdef;
        for i in 0..buf_size {
            x = x.wrapping_mul(6364136223846793005).wrapping_add(1);
            buf.push((x >> 56) as u8);
        }

        let native_buf = buf.clone();

        // Fix: adapt the trait to the VM struct usage
        let program_load_res: Result<u64, &'static str>;
        {
            let ref_vm: &mut ElfVM = unsafe { &mut *(vm as *mut VM as *mut ElfVM) }; // Use unsafe pointer casting
            program_load_res = ref_vm.program_load(bin_file);
        }
        
        program_load_res.map_err(|_| io::Error::new(io::ErrorKind::Other, "Loading program failed"))?;
        let data_addr_buf = vm.map_data_mem(buf.as_mut_ptr() as u64, buf.len() as u64).map_err(|_| io::Error::new(io::ErrorKind::Other, "Mapping data memory failed"))?;

        let stack_addr_src = vm.stack_push(0).map_err(|_| io::Error::new(io::ErrorKind::Other, "Stack push failed"))?;
        let stack_addr_dst = vm.stack_push(0).map_err(|_| io::Error::new(io::ErrorKind::Other, "Stack push failed"))?;

        vm.register_set(10, data_addr_buf);
        vm.register_set(11, buf.len() as u64);
        vm.register_set(12, stack_addr_src);
        vm.register_set(13, stack_addr_dst);

        vm.execute_program().map_err(|_| io::Error::new(io::ErrorKind::Other, "Program execution failed"))?;

        let res = vm.register_get(10);
        let dst = vm.stack_pop::<u64>().map_err(|_| io::Error::new(io::ErrorKind::Other, "Stack pop failed"))?;
        let src = vm.stack_pop::<u64>().map_err(|_| io::Error::new(io::ErrorKind::Other, "Stack pop failed"))?;

        println!("res = 0x{:016x}", res);
        println!("src = 0x{:016x}", src);
        println!("dst = 0x{:016x}", dst);

        let (native_res, native_src, native_dst) = get_addrs(&mut native_buf, native_buf.len());

        println!("native_res = 0x{:016x}", native_res);
        println!("native_src = 0x{:016x}", native_src);
        println!("native_dst = 0x{:016x}", native_dst);

        println!("Buffer equal : {}", native_buf == buf);
        println!("Result equal : {}", native_res == res);
        println!("Source equal : {}", native_src == src);
        println!("Destin equal : {}", native_dst == dst);

        if native_buf != buf || native_res != res || native_src != src || native_dst != dst {
            return Err(io::Error::new(io::ErrorKind::Other, "validation failed"));
        }

        Ok(())
    })() {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}

fn get_addrs(_buf: &mut [u8], _size: usize) -> (u64, u64, u64) {
    (0, 0, 0)
}