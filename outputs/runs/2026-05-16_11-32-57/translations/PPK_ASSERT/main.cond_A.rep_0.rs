use std::vec::Vec;

// Dummy assert macros; replace or implement as needed
macro_rules! assert {
    ($cond:expr) => {
        if !$cond {
            panic!("Assertion failed: {}", stringify!($cond));
        }
    };
    ($cond:expr, $msg:expr) => {
        if !$cond {
            panic!("{}", $msg);
        }
    };
    ($cond:expr, $msg:expr, $($arg:tt)+) => {
        if !$cond {
            panic!($msg, $($arg)+);
        }
    };
}

macro_rules! assert_debug {
    ($cond:expr, $msg:expr, $($arg:tt)+) => {
        if !$cond {
            panic!($msg, $($arg)+);
        }
    };
}

macro_rules! assert_error {
    ($cond:expr, $msg:expr) => {
        if !$cond {
            panic!("{}", $msg);
        }
    };
}

macro_rules! assert_custom {
    ($level:expr, $cond:expr, $msg:expr) => {
        if !$cond {
            panic!("level {}: {}", $level, $msg);
        }
    };
}

struct AssertionException {
    file: &'static str,
    line: u32,
    function: &'static str,
    expression: &'static str,
    what: String,
}

impl AssertionException {
    fn new(file: &'static str, line: u32, function: &'static str, expression: &'static str, what: &str) -> Self {
        Self {
            file,
            line,
            function,
            expression,
            what: what.to_string(),
        }
    }
}

fn trigger_assert_debug_even(i: i32) {
    assert_debug!((i % 2) == 0, "not an even number: {}", i);
}

fn trigger_assert_debug_odd(i: i32) {
    assert_debug!((i % 2) != 0, "not an odd number: {}", i);
}

fn trigger_assert_error() -> Result<(), AssertionException> {
    let ptr: *const () = std::ptr::null();
    if ptr.is_null() {
        return Err(AssertionException::new(
            "file.rs",
            42,
            "trigger_assert_error",
            "ptr != 0",
            "invalid ptr: must not be null",
        ));
    }
    Ok(())
}

fn trigger_assert_custom1() {
    let ptr: *const () = std::ptr::null();
    assert_custom!(100, !ptr.is_null(), "invalid ptr: must not be null");
}

fn trigger_assert_unused_return_value1() -> Vec<i32> {
    vec![0; 10]
}

fn trigger_assert_custom2() {
    let ptr: *const () = std::ptr::null();
    assert_custom!(100, !ptr.is_null(), "invalid ptr: must not be null");
}

fn trigger_assert_unused_return_value2() -> i32 {
    0
}

fn main() {
    assert!(true);
    assert!(true);
    assert!(true);
    assert!(true);
    assert!(true);
    assert_custom!(0, true, "");

    for i in 0..5 {
        trigger_assert_debug_even(i);
    }

    for i in 0..5 {
        trigger_assert_debug_odd(i);
    }

    if let Err(e) = trigger_assert_error() {
        println!("AssertionException caught:");
        println!("  [file]:       {}", e.file);
        println!("  [line]:       {}", e.line);
        println!("  [function]:   {}", e.function);
        println!("  [expression]: {}", e.expression);
        println!("  [what]:       {}", e.what);
        println!();
    }

    trigger_assert_custom1();
    trigger_assert_custom2();

    {
        let mut v = trigger_assert_unused_return_value1();
        v.clear();

        trigger_assert_unused_return_value1();
    }

    trigger_assert_unused_return_value2();

    assert!(false, "He's dead. He's dead, Jim");

    println!();
    println!("if you see this message, this means you decided to ignore all assertions");
}