use std::vec::Vec;

macro_rules! assert_custom {
    ($code:expr, $cond:expr $(, $msg:expr $(, $arg:expr)*)?) => {
        if !$cond {
            panic!($($msg $(, $arg)*)?);
        }
    };
}

macro_rules! assert_debug {
    ($cond:expr, $msg:expr $(, $arg:expr)*) => {
        debug_assert!($cond, $msg $(, $arg)*);
    };
}

macro_rules! assert_error {
    ($cond:expr, $msg:expr $(, $arg:expr)*) => {
        assert!($cond, $msg $(, $arg)*);
    };
}

macro_rules! assert_fatal {
    ($cond:expr, $msg:expr $(, $arg:expr)*) => {
        if !$cond {
            panic!($msg $(, $arg)*);
        }
    };
}

#[inline(always)]
fn trigger_assert_debug_even(i: i32) {
    assert_debug!((i % 2) == 0, "not an even number: {}", i);
}

#[inline(always)]
fn trigger_assert_debug_odd(i: i32) {
    assert_debug!((i % 2) != 0, "not an odd number: {}", i);
}

fn trigger_assert_error() {
    let ptr: *const u8 = std::ptr::null();
    assert_error!(!ptr.is_null(), "invalid ptr: must not be null");
}

fn trigger_assert_custom1() {
    let ptr: *const u8 = std::ptr::null();
    assert_custom!(100, !ptr.is_null(), "invalid ptr: must not be null");
}

#[must_use]
fn trigger_assert_unused_return_value1() -> Vec<i32> {
    vec![0; 10]
}

fn trigger_assert_custom2() {
    let ptr: *const u8 = std::ptr::null();
    assert_custom!(100, !ptr.is_null(), "invalid ptr: must not be null");
}

#[must_use]
fn trigger_assert_unused_return_value2() -> i32 {
    0
}

fn main() {
    assert!(true);
    assert!(true);
    debug_assert!(true);
    assert!(true);
    if !true {
        panic!();
    }
    assert_custom!(0, true);

    for i in 0..5 {
        trigger_assert_debug_even(i);
    }

    for i in 0..5 {
        trigger_assert_debug_odd(i);
    }

    let assert_err_result = std::panic::catch_unwind(|| trigger_assert_error());
    if let Err(err) = assert_err_result {
        println!("AssertionException caught:");
        match *err.downcast_ref::<String>().unwrap() {
            _ => {
                println!("  [file]: filename.rs");
                println!("  [line]: line_number");
                println!("  [function]: function_name");
                println!("  [expression]: expression");
                println!("  [what]: error_description");
                println!();
            },
        }
    }

    trigger_assert_custom1();
    trigger_assert_custom2();

    {
        let mut v = trigger_assert_unused_return_value1();
        v.clear();

        // trigger assert on scope exit
        let _ = trigger_assert_unused_return_value1();
    }

    {
        let _ = trigger_assert_unused_return_value2();
    }

    assert_fatal!(false, "He's dead. He's dead, Jim");

    println!();
    println!("if you see this message, this means you decided to ignore all assertions");
}