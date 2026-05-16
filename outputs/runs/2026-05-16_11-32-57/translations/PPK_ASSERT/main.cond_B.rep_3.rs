use std::fmt;

// Placeholder assert macros
macro_rules! assert_custom {
    ($code:expr, $condition:expr, $($arg:tt)*) => {
        if !$condition {
            eprintln!("Custom assert failed (code: {}): ", $code);
            panic!($($arg)*);
        }
    };
}

macro_rules! assert {
    ($condition:expr, $($arg:tt)*) => {
        if !$condition {
            eprintln!("Assert failed: ");
            panic!($($arg)*);
        }
    };
}

macro_rules! assert_warning {
    ($condition:expr) => {
        if !$condition {
            eprintln!("Warning: Condition failed.");
        }
    };
}

macro_rules! assert_debug {
    ($condition:expr, $($arg:tt)*) => {
        if cfg!(debug_assertions) && !$condition {
            eprintln!("Debug assert failed: ");
            panic!($($arg)*);
        }
    };
}

macro_rules! assert_error {
    ($condition:expr, $($arg:tt)*) => {
        if !$condition {
            eprintln!("Error assert failed: ");
            panic!($($arg)*);
        }
    };
}

macro_rules! assert_fatal {
    ($condition:expr, $($arg:tt)*) => {
        if !$condition {
            eprintln!("Fatal assert failed: ");
            panic!($($arg)*);
        }
    };
}

fn trigger_assert_debug_even(i: i32) {
    assert_debug!(i % 2 == 0, "not an even number: {}", i);
}

fn trigger_assert_debug_odd(i: i32) {
    assert_debug!(i % 2 != 0, "not an odd number: {}", i);
}

fn trigger_assert_error() {
    let ptr: *const u8 = std::ptr::null();
    assert_error!(!ptr.is_null(), "invalid ptr: must not be null");
}

fn trigger_assert_custom1() {
    let ptr: *const u8 = std::ptr::null();
    assert_custom!(100, !ptr.is_null(), "invalid ptr: must not be null");
}

fn trigger_assert_unused_return_value1() -> Vec<i32> {
    vec![0; 10]
}

fn trigger_assert_custom2() {
    let _enabled = 0;
    let ptr: *const u8 = std::ptr::null();
    // No assertion if PPK_ASSERT_ENABLED is 0
    #[allow(unused_variables)]
    assert_custom!(100, !ptr.is_null(), "invalid ptr: must not be null");
}

#[must_use]
fn trigger_assert_unused_return_value2() -> i32 {
    0
}

fn main() {
    assert!(true, "");
    assert_warning!(true);
    assert_debug!(true, "");
    assert_error!(true, "");
    assert_fatal!(true, "");
    assert_custom!(0, true, "");

    for i in 0..5 {
        trigger_assert_debug_even(i);
    }

    for i in 0..5 {
        trigger_assert_debug_odd(i);
    }

    if let Err(e) = std::panic::catch_unwind(|| {
        trigger_assert_error();
    }) {
        if let Some(e) = e.downcast_ref::<&str>() {
            println!("AssertionException caught:");
            println!("  [what]: {}", e);
            println!();
        }
    }

    trigger_assert_custom1();

    trigger_assert_custom2();

    {
        let mut v = trigger_assert_unused_return_value1();
        v.clear();

        // trigger assert on scope exit
        trigger_assert_unused_return_value1();
    }

    {
        trigger_assert_unused_return_value2();
    }

    assert_fatal!(false, "He's dead. He's dead, Jim");

    println!("\nif you see this message, this means you decided to ignore all assertions");
}