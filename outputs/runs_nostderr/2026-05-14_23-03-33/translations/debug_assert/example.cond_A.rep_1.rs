use std::panic;

mod debug_assert {
    pub struct SourceLocation {
        pub file_name: &'static str,
        pub line_number: u32,
    }

    pub struct DefaultHandler;

    pub struct Level<const L: u32>;

    pub fn debug_assert(condition: bool, location: SourceLocation, expression: &str, handler: impl Fn(&SourceLocation, &str, Option<*const ()>) -> (), ptr: Option<*const ()>) {
        if !condition {
            handler(&location, expression, ptr);
        }
    }

    pub fn debug_unreachable(location: SourceLocation, handler: impl Fn(&SourceLocation, &str)) {
        handler(&location, "unreachable code reached");
    }
}

const MODULE_A_LEVEL: u32 = 1;

struct ModuleA;

impl ModuleA {
    const HANDLER: debug_assert::DefaultHandler = debug_assert::DefaultHandler;

    fn level() -> debug_assert::Level<1> {
        debug_assert::Level
    }
}

fn module_a_func(ptr: Option<*const ()>) {
    debug_assert::debug_assert(!ptr.is_none(),
        debug_assert::SourceLocation { file_name: file!(), line_number: line!() },
        "ptr",
        |loc, expr, _| eprintln!("default handler: assertion failed"),
        ptr
    );

    debug_assert::debug_assert(2 + 2 == 4,
        debug_assert::SourceLocation { file_name: file!(), line_number: line!() },
        "2 + 2 == 4",
        |loc, expr, _| eprintln!("default handler: level 2 assertion passed"),
        None
    );

    debug_assert::debug_assert(false,
        debug_assert::SourceLocation { file_name: file!(), line_number: line!() },
        "1 == 0",
        |loc, expr, _| eprintln!("default handler: assertion with message failed: this should be true"),
        None
    );

    debug_assert::debug_unreachable(
        debug_assert::SourceLocation { file_name: file!(), line_number: line!() },
        |loc, _| eprintln!("default handler: unreachable code reached")
    );
}

const MODULE_B_LEVEL: u32 = 2;

struct ModuleB;

impl ModuleB {
    fn handler(location: &debug_assert::SourceLocation, expression: &str, ptr: Option<*const ()>) {
        eprint!("Assertion failure '{}:{}': {}", location.file_name, location.line_number, expression);
        if let Some(p) = ptr {
            eprint!(" - pointer is {:?}", p);
        }
        eprintln!();
    }

    fn level() -> debug_assert::Level<2> {
        debug_assert::Level
    }
}

fn module_b_func(value: &i32, ptr: *const i32) {
    debug_assert::debug_assert(ptr == value as *const i32,
        debug_assert::SourceLocation { file_name: file!(), line_number: line!() },
        "ptr == &value",
        |loc, expr, ptr_opt| ModuleB::handler(loc, expr, ptr_opt),
        Some(ptr as *const ()),
    );

    debug_assert::debug_assert(ptr == value as *const i32,
        debug_assert::SourceLocation { file_name: file!(), line_number: line!() },
        "ptr == &value",
        |loc, expr, ptr_opt| ModuleB::handler(loc, expr, ptr_opt),
        Some(ptr as *const ()),
    );
}

fn main() {
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(|info| {
        let payload = info.payload().downcast_ref::<&str>().unwrap_or(&"");
        eprintln!("Custom panic handler: {}", payload);
        std::process::exit(0);
    }));

    module_a_func(None);
    let val = 5;
    module_b_func(&val, &val as *const _);

    panic::set_hook(original_hook);
}