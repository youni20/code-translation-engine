use std::process;

mod module_a {
    pub struct ModuleA;

    pub fn module_a_func(ptr: Option<&mut u8>) {
        if ptr.is_some() {
            println!("Pointer is Some.");
        } else {
            println!("Pointer is None.");
        }
        
        if 2 + 2 == 4 {
            println!("2 + 2 equals 4.");
        }
        
        // simulate unreachable with panic
        panic!("Reached unreachable code!");
    }
}

mod module_b {
    use std::io::{self, Write};

    pub struct ModuleB;

    impl ModuleB {
        pub fn handle(
            loc: &str,
            line_number: u32,
            expression: &str,
            ptr: Option<*const u8>,
        ) {
            let stderr = io::stderr();
            let mut handle = stderr.lock();
            let _ = write!(
                &mut handle,
                "Assertion failure '{}:{}: {}'",
                loc, line_number, expression
            );
            if let Some(p) = ptr {
                let _ = write!(&mut handle, " - pointer is {:?}", p);
            }
            let _ = writeln!(&mut handle);
        }
    }

    pub fn module_b_func(value: &mut i32, ptr: Option<*mut i32>) {
        if ptr == Some(value as *mut i32) {
            println!("Pointer is equal to value.");
        } else {
            println!("Pointer is not equal to value.");
        }
    }
}

fn main() {
    module_a::module_a_func(None);
    let mut val = 5;
    let val_ptr = &mut val as *mut i32; // Create a raw pointer outside the function call
    module_b::module_b_func(&mut val, Some(val_ptr));
}

extern "C" fn signal_handler(signal: i32) {
    if signal == 6 { // SIGABRT is usually 6 on Unix-based systems
        eprintln!("Please never call std::abort() in production :)");
        process::exit(0);
    }
}