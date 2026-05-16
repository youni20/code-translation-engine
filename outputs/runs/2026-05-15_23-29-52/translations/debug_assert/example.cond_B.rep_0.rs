use std::ffi::c_void;
use std::io::{self, Write};
use std::process;
use std::ptr;
use std::sync::atomic::{AtomicPtr, Ordering};

// Simulate libc signals and handlers with atomic pointers
static SIGNAL_HANDLER_PTR: AtomicPtr<c_void> = AtomicPtr::new(ptr::null_mut());

mod debug_assert {
    #[derive(Default)]
    pub struct DefaultHandler;

    pub struct SetLevel<const LEVEL: usize>;

    pub struct Level<const N: usize>;

    pub fn debug_assert(predicate: bool, handler: impl Fn(), _level: usize) {
        if !predicate {
            handler();
        }
    }

    pub struct SourceLocation {
        pub file_name: &'static str,
        pub line_number: u32,
    }

    pub fn unreachable(handler: impl Fn()) {
        handler();
    }

    impl Default for SourceLocation {
        fn default() -> Self {
            SourceLocation {
                file_name: "unknown",
                line_number: 0,
            }
        }
    }
}

//=== module A ===//
const MODULE_A_LEVEL: usize = 1;

#[derive(Default)]
struct ModuleA;

impl ModuleA {
    fn handle() {
        eprintln!("Assertion in module A failed");
    }
}

fn module_a_func(ptr: *const c_void) {
    debug_assert::debug_assert(!ptr.is_null(), ModuleA::handle, MODULE_A_LEVEL);
    debug_assert::debug_assert(2 + 2 == 4, ModuleA::handle, 2);
    debug_assert::debug_assert(1 == 0, ModuleA::handle, MODULE_A_LEVEL); // This assertion will fail
    debug_assert::unreachable(ModuleA::handle);
}

//=== module B ===//
const MODULE_B_LEVEL: usize = 2;

struct ModuleB;

impl ModuleB {
    fn handle(loc: &debug_assert::SourceLocation, expression: &str, ptr: Option<*const std::ffi::c_void>) {
        write!(io::stderr(),
            "Assertion failure '{}:{}: {}",
            loc.file_name, loc.line_number, expression,
        ).unwrap();
        if let Some(ptr) = ptr {
            write!(io::stderr(), " - pointer is {:?}", ptr).unwrap();
        }
        writeln!(io::stderr()).unwrap();
    }
}

fn module_b_func(value: &mut i32, ptr: *const c_void) {
    debug_assert::debug_assert(ptr == value as *mut i32 as *const c_void, || {
        ModuleB::handle(&debug_assert::SourceLocation::default(), "ptr == value", Some(ptr))
    }, MODULE_B_LEVEL);
    debug_assert::debug_assert(ptr == value as *mut i32 as *const c_void, || {
        ModuleB::handle(&debug_assert::SourceLocation::default(), "ptr == value", Some(ptr))
    }, 2);
}

fn main() {
    let signal_handler_ptr = signal_handler as *mut c_void;
    SIGNAL_HANDLER_PTR.store(signal_handler_ptr, Ordering::SeqCst);

    module_a_func(ptr::null());
    let mut val = 5;
    let val_ptr = &val as *const i32 as *const c_void;
    module_b_func(&mut val, val_ptr);
}

extern "C" fn signal_handler(_signal: i32) {
    eprintln!("Please never call std::abort() in production :)");
    process::exit(0);
}