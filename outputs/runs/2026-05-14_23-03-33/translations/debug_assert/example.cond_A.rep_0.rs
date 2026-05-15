use std::process;
use std::panic;
use std::sync::Once;
use std::os::raw::c_void;

// Simulate a debug_assert library in Rust
mod debug_assert {
    use super::*;
    
    pub struct SourceLocation {
        pub file_name: &'static str,
        pub line_number: u32,
    }

    pub struct DefaultHandler;

    pub trait Level {
        fn level() -> usize;
    }

    impl Level for Level1 {
        fn level() -> usize { 1 }
    }

    impl Level for Level2 {
        fn level() -> usize { 2 }
    }

    pub struct Level1;

    pub struct Level2;

    pub trait SetLevel<L: Level> {
        fn current_level() -> usize {
            L::level()
        }
    }

    pub struct Assert<L: Level, H> {
        level: L,
        handler: H,
    }

    pub fn assert<L: Level, H>(condition: bool, handler: H, _message: Option<&str>)
    where
        H: Fn(&SourceLocation, &str, Option<*mut c_void>),
    {
        if !condition {
            let loc = SourceLocation {
                file_name: file!(),
                line_number: line!(),
            };
            handler(&loc, "Assertion failed", None);
            panic!("Assertion failed");
        }
    }
    
    pub fn unreachable<H>(handler: H)
    where
        H: Fn(&SourceLocation),
    {
        let loc = SourceLocation {
            file_name: file!(),
            line_number: line!(),
        };
        handler(&loc);
        panic!("Reached unreachable code");
    }
}

//=== module A ===//
const MODULE_A_LEVEL: usize = 1;

// tag type that defines a module 
struct ModuleA;
impl debug_assert::SetLevel<debug_assert::Level1> for ModuleA {}

impl ModuleA {
    fn handle(loc: &debug_assert::SourceLocation, expression: &str, _ptr: Option<*mut c_void>) {
        eprintln!("Assertion failure '{}' at {}:{}", expression, loc.file_name, loc.line_number);
    }

    fn handle_unreachable(loc: &debug_assert::SourceLocation) {
        eprintln!("Unreachable code at {}:{}", loc.file_name, loc.line_number);
    }
}

fn module_a_func(ptr: Option<*mut c_void>) {
    debug_assert::assert::<debug_assert::Level1, _>(ptr.is_some(), ModuleA::handle, None);
    debug_assert::assert::<debug_assert::Level1, _>(2 + 2 == 4, ModuleA::handle, None);
    debug_assert::assert::<debug_assert::Level1, _>(false, ModuleA::handle, Some("This should be true"));
    debug_assert::unreachable(ModuleA::handle_unreachable);
}

//=== module B ===//
const MODULE_B_LEVEL: usize = 2;

struct ModuleB;
impl debug_assert::SetLevel<debug_assert::Level2> for ModuleB {}

impl ModuleB {
    fn handle(loc: &debug_assert::SourceLocation, expression: &str, ptr: Option<*mut c_void>) {
        eprint!("Assertion failure '{}' at {}:{}", expression, loc.file_name, loc.line_number);
        if let Some(ptr_val) = ptr {
            eprint!(" - pointer is {:?}", ptr_val);
        }
        eprintln!();
    }
}

fn module_b_func(value: &i32, ptr: Option<*mut c_void>) {
    debug_assert::assert::<debug_assert::Level2, _>(ptr.map_or(false, |p| p == (value as *const i32 as *mut c_void)), ModuleB::handle, None);
    debug_assert::assert::<debug_assert::Level2, _>(ptr.map_or(false, |p| p == (value as *const i32 as *mut c_void)), ModuleB::handle, None);
}

fn main() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        panic::set_hook(Box::new(|info| {
            if info.payload().is::<&str>() && *info.payload().downcast_ref::<&str>().unwrap() == "Assertion failed" {
                eprintln!("Please never call std::abort() in production :)");
                process::exit(0);
            }
        }));
    });
    
    module_a_func(None);
    let val = 5;
    module_b_func(&val, Some(&val as *const _ as *mut c_void));
}