use std::ffi::c_void;
use std::ptr;
use std::process;
use std::panic;
use std::ops::Not;

//=== module A ===//
const MODULE_A_LEVEL: i32 = 1;

// Tag type that defines a module
struct ModuleA;
impl ModuleA {
    fn assert_level() -> i32 {
        MODULE_A_LEVEL
    }

    fn default_handler(expr: &str, location: &DebugLocation, _msg: Option<&str>) {
        eprintln!("Assertion failed '{}:{}': {}", location.file_name, location.line_number, expr);
        process::abort();
    }
}

fn module_a_func(ptr: *mut c_void) {
    debug_assert(ptr.is_null().not(), ModuleA, Option::<&str>::None);
    debug_assert(2 + 2 == 4, ModuleA, Some("2 + 2 should equal 4"));
    debug_assert(false, ModuleA, Some("this should be true"));
    debug_unreachable(ModuleA);
}

//=== module B ===//
const MODULE_B_LEVEL: i32 = 2;

struct ModuleB;
impl ModuleB {
    fn assert_level() -> i32 {
        MODULE_B_LEVEL
    }

    // Module B uses a different handler
    // It does not support a message, instead, you can specify a pointer value
    fn handle(expr: &str, location: &DebugLocation, ptr: Option<*mut c_void>) {
        eprint!("Assertion failure '{}:{}': {}", location.file_name, location.line_number, expr);
        if let Some(ptr) = ptr {
            eprint!(" - pointer is {:?}", ptr);
        }
        eprintln!();
        process::abort();
    }
}

fn module_b_func(value: &mut i32, ptr: *mut c_void) {
    debug_assert(ptr == value as *mut _ as *mut c_void, ModuleB, None);
    debug_assert(ptr == value as *mut _ as *mut c_void, ModuleB, None);
}

struct DebugLocation {
    file_name: &'static str,
    line_number: u32,
}

fn debug_assert<M: DebugModule>(
    condition: bool,
    _module: M,
    msg: Option<&str>,
) {
    if !condition && M::assert_level() > 0 {
        M::default_handler(
            "Assertion failed",
            &DebugLocation {
                file_name: file!(),
                line_number: line!(),
            },
            msg,
        );
    }
}

fn debug_unreachable<M: DebugModule>(_module: M) {
    M::default_handler(
        "Entered unreachable code",
        &DebugLocation {
            file_name: file!(),
            line_number: line!(),
        },
        None,
    );
}

trait DebugModule {
    fn assert_level() -> i32;
    fn default_handler(expr: &str, location: &DebugLocation, msg: Option<&str>);
}

impl DebugModule for ModuleA {
    fn assert_level() -> i32 {
        ModuleA::assert_level()
    }

    fn default_handler(expr: &str, location: &DebugLocation, msg: Option<&str>) {
        ModuleA::default_handler(expr, location, msg)
    }
}

impl DebugModule for ModuleB {
    fn assert_level() -> i32 {
        ModuleB::assert_level()
    }

    fn default_handler(expr: &str, location: &DebugLocation, _msg: Option<&str>) {
        ModuleB::handle(expr, location, None)
    }
}

fn main() {
    let prev_hook = panic::take_hook();
    panic::set_hook(Box::new(|_info| {
        eprintln!("Please never panic! in production :)");
        process::exit(0);
    }));

    module_a_func(ptr::null_mut());
    let mut val = 5;
    let ptr_to_val = &mut val as *mut _ as *mut c_void;
    module_b_func(&mut val, ptr_to_val);

    panic::set_hook(prev_hook);
}