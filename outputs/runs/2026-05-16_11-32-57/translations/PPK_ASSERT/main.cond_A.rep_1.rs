use std::collections::HashMap;
use std::fmt;

macro_rules! assert_custom {
    ($condition:expr, $message:expr) => {
        if !$condition {
            panic!("{}", $message);
        }
    };
    ($level:expr, $condition:expr, $message:expr) => {
        if !$condition {
            panic!("Level {}: {}", $level, $message);
        }
    };
}

macro_rules! assert_warning {
    ($condition:expr, $($arg:tt)+) => {
        if !$condition {
            eprintln!($($arg)+);
        }
    };
}

macro_rules! assert_debug {
    ($condition:expr, $($arg:tt)+) => {
        if !$condition {
            eprintln!($($arg)+);
        }
    };
}

macro_rules! assert_error {
    ($condition:expr, $($arg:tt)+) => {
        if !$condition {
            panic!($($arg)+);
        }
    };
}

macro_rules! assert_fatal {
    ($condition:expr, $($arg:tt)+) => {
        if !$condition {
            panic!($($arg)+);
        }
    };
}

macro_rules! assert_used {
    ($expr:expr) => {
        $expr
    };
}

fn trigger_assert_debug_even(i: i32) {
    assert_debug!((i % 2) == 0, "not an even number: {}", i);
}

fn trigger_assert_debug_odd(i: i32) {
    assert_debug!((i % 2) != 0, "not an odd number: {}", i);
}

fn trigger_assert_error() {
    let ptr: *const i32 = std::ptr::null();
    assert_error!(!ptr.is_null(), "invalid ptr: must not be null");
}

fn trigger_assert_custom1() {
    let ptr: *const i32 = std::ptr::null();
    assert_custom!(100, !ptr.is_null(), "invalid ptr: must not be null");
}

fn trigger_assert_unused_return_value1() -> Vec<i32> {
    vec![0; 10]
}

fn trigger_assert_custom2() {
    let ptr: *const i32 = std::ptr::null();
    assert_custom!(100, !ptr.is_null(), "invalid ptr: must not be null");
}

fn trigger_assert_unused_return_value2() -> i32 {
    0
}

fn main() {
    assert_custom!(true, "Assert");
    assert_warning!(true, "Warning");
    assert_debug!(true, "Debug");
    assert_error!(true, "Error");
    assert_fatal!(true, "Fatal");
    assert_custom!(0, true, "Custom");

    for i in 0..5 {
        trigger_assert_debug_even(i);
    }

    for i in 0..5 {
        trigger_assert_debug_odd(i);
    }

    let res = std::panic::catch_unwind(|| {
        trigger_assert_error();
    });

    if res.is_err() {
        eprintln!("AssertionException caught");
    }

    trigger_assert_custom1();

    trigger_assert_custom2();

    {
        let mut v = trigger_assert_unused_return_value1();
        v.clear();

        // trigger assert on scope exit
        assert_used!(trigger_assert_unused_return_value1());
    }

    trigger_assert_unused_return_value2();

    assert_fatal!(false, "He's dead. He's dead, Jim");

    println!();
    println!("if you see this message, this means you decided to ignore all assertions");
}