use std::ffi::c_void;
use std::ptr;
use std::io::{self, Write};

// Assuming hypothetical debug_assert module as in C++ code
mod debug_assert {
    use std::ffi::c_void;

    pub struct SourceLocation {
        pub file_name: &'static str,
        pub line_number: u32,
    }

    pub struct DebugAssert;

    impl DebugAssert {
        pub fn assert(expr: bool, _handler: impl Handlable, _level: Option<u32>, _msg: Option<&str>, _ptr: Option<*const c_void>) {
            if !expr {
                eprintln!("Assertion failed");
            }
        }

        pub fn unreachable(_handler: impl Handlable) {
            eprintln!("Unreachable code reached");
        }
    }

    pub trait Handlable {
        fn handle(&self, loc: &SourceLocation, expression: &str, ptr: Option<*const c_void>);
    }

    pub struct DefaultHandler;

    impl Handlable for DefaultHandler {
        fn handle(&self, _loc: &SourceLocation, _expression: &str, _ptr: Option<*const c_void>) {
            // Default handler does nothing
        }
    }

    pub struct Level<const N: u32>;

    pub mod level {
        use super::Level;
        pub const LEVEL_2: Level<2> = Level;
    }
}

//=== module A ===//
const MODULE_A_LEVEL: u32 = 1;

struct ModuleA;

impl debug_assert::Handlable for ModuleA {
    fn handle(&self, loc: &debug_assert::SourceLocation, expression: &str, _ptr: Option<*const c_void>) {
        eprintln!("Assertion failure: '{}' at {}:{}", expression, loc.file_name, loc.line_number);
    }
}

fn module_a_func(ptr: *mut c_void) {
    const LOC: debug_assert::SourceLocation = debug_assert::SourceLocation {
        file_name: file!(),
        line_number: line!(),
    };

    debug_assert::DebugAssert::assert(!ptr.is_null(), ModuleA, None, None, None);
    debug_assert::DebugAssert::assert(2 + 2 == 4, ModuleA, Some(2), None, None);
    debug_assert::DebugAssert::assert(1 == 0, ModuleA, None, Some("this should be true"), None);
    debug_assert::DebugAssert::unreachable(ModuleA);
}

//=== module B ===//
const MODULE_B_LEVEL: u32 = 2;

struct ModuleB;

impl debug_assert::Handlable for ModuleB {
    fn handle(&self, loc: &debug_assert::SourceLocation, expression: &str, ptr: Option<*const c_void>) {
        let stderr = io::stderr();
        let mut handle = stderr.lock();
        writeln!(handle, "Assertion failure '{}:{}: {}", loc.file_name, loc.line_number, expression).unwrap();

        if let Some(ptr) = ptr {
            writeln!(handle, " - pointer is {:?}", ptr).unwrap();
        }
    }
}

fn module_b_func(value: &mut i32, ptr: *mut c_void) {
    const LOC: debug_assert::SourceLocation = debug_assert::SourceLocation {
        file_name: file!(),
        line_number: line!(),
    };

    debug_assert::DebugAssert::assert(ptr as *mut i32 == value, ModuleB, None, None, Some(ptr));
    debug_assert::DebugAssert::assert(ptr as *mut i32 == value, ModuleB, Some(2), None, Some(ptr));
}

fn main() {
    module_a_func(ptr::null_mut());

    let mut val = 5;
    {
        let val_ptr = &mut val as *mut _ as *mut c_void;
        module_b_func(&mut val, val_ptr);
    }
}