use std::ffi::c_void;
use std::ptr;
use std::sync::Once;
use std::panic;
use std::process;
use std::io::{self, Write};

//=== module A ===//
const MODULE_A_LEVEL: usize = 1;

struct ModuleA;

impl ModuleA {
    fn level() -> usize {
        MODULE_A_LEVEL
    }
}

fn module_a_func(ptr: *mut c_void) {
    debug_assert!(ptr != ptr::null_mut(), "Module A assertion failed"); // minimal assertion
    assert_eq!(2 + 2, 4, "Module A assertion failed with level 2"); // assertion with level
    assert!(1 == 0, "this should be true"); // assertion with additional parameters, i.e. a message
    unreachable!("Module A unreachable statement"); // mark unreachable statements
}

//=== module B ===//
const MODULE_B_LEVEL: usize = 2;

struct ModuleB;

impl ModuleB {
    fn level() -> usize {
        MODULE_B_LEVEL
    }
    
    fn handle(expression: &str, ptr: Option<*const c_void>) {
        let stderr = io::stderr();
        let mut handle = stderr.lock();
        let _ = write!(handle, "Assertion failure: {}\n", expression);
        if let Some(ptr_val) = ptr {
            let _ = write!(handle, " - pointer is {:p}\n", ptr_val);
        }
    }
}

fn module_b_func(value: &mut i32, ptr: *mut c_void) {
    let val_ptr = value as *mut _ as *mut c_void;
    if ptr != val_ptr { 
        ModuleB::handle("ptr == &value", Some(ptr)); 
    }
    debug_assert_eq!(ptr, val_ptr, "Module B assertion failed with level 2");
}

fn main() {
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        panic::set_hook(Box::new(|_info| {
            eprintln!("Please never call std::abort() in production :)");
            process::exit(0);
        }));
    });

    module_a_func(ptr::null_mut());
    let mut val = 5;
    {
        let ptr = &mut val as *mut _ as *mut c_void;
        module_b_func(&mut val, ptr);
    }
}