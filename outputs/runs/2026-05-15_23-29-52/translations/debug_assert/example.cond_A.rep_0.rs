use std::ptr;
use std::os::raw::{c_void};

const MODULE_A_LEVEL: u8 = 1;
const MODULE_B_LEVEL: u8 = 2;

#[derive(Default)]
struct ModuleATag;

struct ModuleBTag;

struct DebugAssert {
    level: u8,
}

impl DebugAssert {
    fn level(&self) -> u8 {
        self.level
    }

    fn assert(condition: bool, module: impl DebugAssertLevel, assert_level: u8, message: &str, ptr: Option<*const c_void>) {
        if module.level() >= assert_level && !condition {
            eprintln!("Assertion failed: {}", message);
            if let Some(ptr) = ptr {
                eprintln!(" - pointer is {:?}", ptr);
            }
            panic!("Aborting due to failed assertion");
        }
    }
}

trait DebugAssertLevel {
    fn level(&self) -> u8;
}

impl DebugAssertLevel for ModuleATag {
    fn level(&self) -> u8 {
        MODULE_A_LEVEL
    }
}

impl DebugAssertLevel for ModuleBTag {
    fn level(&self) -> u8 {
        MODULE_B_LEVEL
    }
}

fn module_a_func(ptr: *mut c_void) {
    DebugAssert::assert(!ptr.is_null(), ModuleATag, 1, "ptr must not be null", None);
    DebugAssert::assert(2 + 2 == 4, ModuleATag, 2, "2 + 2 should equal 4", None);
    DebugAssert::assert(1 == 0, ModuleATag, 1, "this should be true", None);
    unreachable!("Marked unreachable by module_a");
}

fn module_b_func(value: &mut i32, ptr: *mut c_void) {
    DebugAssert::assert(ptr == value as *mut _ as *mut c_void, ModuleBTag, 1, "ptr must equal value pointer", Some(ptr as *const _));
    DebugAssert::assert(ptr == value as *mut _ as *mut c_void, ModuleBTag, 2, "ptr must equal value pointer", Some(ptr as *const _));
}

fn main() {
    module_a_func(ptr::null_mut());
    let mut val = 5;
    let val_ptr = &mut val as *mut _ as *mut c_void;
    module_b_func(&mut val, val_ptr);
}