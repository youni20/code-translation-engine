use std::collections::VecDeque;

macro_rules! assert_true {
    ($expr:expr, $msg:expr) => {
        if !$expr {
            eprintln!("Assertion failed: {}", $msg);
            panic!();
        }
    };
    ($expr:expr, $msg:expr, $($arg:tt)*) => {
        if !$expr {
            eprintln!(concat!("Assertion failed: ", $msg), $($arg)*);
            panic!();
        }
    }
}

fn trigger_assert_debug_even(i: i32) {
    assert_true!((i % 2) == 0, "not an even number: {}", i);
}

fn trigger_assert_debug_odd(i: i32) {
    assert_true!((i % 2) != 0, "not an odd number: {}", i);
}

fn trigger_assert_error() {
    let ptr: *const () = std::ptr::null();
    assert_true!(!ptr.is_null(), "invalid ptr: must not be null");
}

fn trigger_assert_custom1() {
    let ptr: *const () = std::ptr::null();
    assert_true!(!ptr.is_null(), "invalid ptr: must not be null");
}

fn trigger_assert_unused_return_value1() -> VecDeque<i32> {
    VecDeque::from(vec![0; 10])
}

fn trigger_assert_custom2() {
    let ptr: *const () = std::ptr::null();
    assert_true!(!ptr.is_null(), "invalid ptr: must not be null");
}

fn trigger_assert_unused_return_value2() -> i32 {
    0
}

fn main() {
    assert_true!(true, "General test");
    assert_true!(true, "Warning test");
    assert_true!(true, "Debug test");
    assert_true!(true, "Error test");
    assert_true!(true, "Fatal test");
    assert_true!(true, "Custom test");

    for i in 0..5 {
        trigger_assert_debug_even(i);
    }

    for i in 0..5 {
        trigger_assert_debug_odd(i);
    }

    match std::panic::catch_unwind(|| trigger_assert_error()) {
        Ok(_) => (),
        Err(_) => {
            eprintln!("AssertionException caught:");
            eprintln!("Custom error handling... (original C++ lines/fields not available)");
        }
    }

    trigger_assert_custom1();
    trigger_assert_custom2();

    {
        let mut v = trigger_assert_unused_return_value1();
        v.clear();

        trigger_assert_unused_return_value1();
    }

    {
        trigger_assert_unused_return_value2();
    }

    assert_true!(false, "He's dead. He's dead, Jim");

    println!();
    println!("if you see this message, this means you decided to ignore all assertions");
}