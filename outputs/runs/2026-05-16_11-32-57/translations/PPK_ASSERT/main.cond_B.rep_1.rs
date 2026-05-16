use std::fmt;

struct AssertionError {
    file: &'static str,
    line: u32,
    function: &'static str,
    expression: &'static str,
    what: String,
}

impl AssertionError {
    fn new(file: &'static str, line: u32, function: &'static str, expression: &'static str, what: &str) -> Self {
        AssertionError {
            file,
            line,
            function,
            expression,
            what: what.to_string(),
        }
    }
}

impl fmt::Display for AssertionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[file]: {}\n[line]: {}\n[function]: {}\n[expression]: {}\n[what]: {}\n", self.file, self.line, self.function, self.expression, self.what)
    }
}

macro_rules! ASSERT_DEBUG {
    ($cond:expr, $msg:expr, $($arg:tt)*) => {
        if !$cond {
            panic!("Debug Assertion failed: {}", format!($msg, $($arg)*));
        }
    };
}

macro_rules! ASSERT_ERROR {
    ($cond:expr, $msg:expr) => {
        if !$cond {
            panic!("Error Assertion failed: {}", $msg);
        }
    };
}

macro_rules! ASSERT_CUSTOM {
    ($code:expr, $cond:expr, $msg:expr) => {
        if !$cond {
            panic!("Custom[{}] Assertion failed: {}", $code, $msg);
        }
    };
}

macro_rules! ASSERT_USED {
    ($ret:ty) => {};
}

fn trigger_assert_debug_even(i: i32) {
    ASSERT_DEBUG!(i % 2 == 0, "not an even number: {}", i);
}

fn trigger_assert_debug_odd(i: i32) {
    ASSERT_DEBUG!(i % 2 != 0, "not an odd number: {}", i);
}

fn trigger_assert_error() {
    let ptr: *const u8 = std::ptr::null();
    ASSERT_ERROR!(!ptr.is_null(), "invalid ptr: must not be null");
}

fn trigger_assert_custom1() {
    let ptr: *const u8 = std::ptr::null();
    ASSERT_CUSTOM!(100, !ptr.is_null(), "invalid ptr: must not be null");
}

fn trigger_assert_unused_return_value1() -> Vec<i32> {
    vec![0; 10]
}

fn trigger_assert_custom2() {
    let ptr: *const u8 = std::ptr::null();
    ASSERT_CUSTOM!(100, !ptr.is_null(), "invalid ptr: must not be null");
}

fn trigger_assert_unused_return_value2() -> i32 {
    0
}

fn main() {
    let assertions_enabled = true;
    if assertions_enabled {
        for i in 0..5 {
            trigger_assert_debug_even(i);
        }

        for i in 0..5 {
            trigger_assert_debug_odd(i);
        }

        let _ = std::panic::catch_unwind(|| {
            trigger_assert_error();
        }).map_err(|e| {
            if let Some(e) = e.downcast_ref::<AssertionError>() {
                println!("AssertionException caught:");
                println!("{}", e);
            }
        });

        trigger_assert_custom1();
        trigger_assert_custom2();

        let mut v = trigger_assert_unused_return_value1();
        v.clear();

        trigger_assert_unused_return_value1();
        trigger_assert_unused_return_value2();

        ASSERT_CUSTOM!(0, false, "He's dead. He's dead, Jim");
    }

    println!("\nif you see this message, this means you decided to ignore all assertions");
}