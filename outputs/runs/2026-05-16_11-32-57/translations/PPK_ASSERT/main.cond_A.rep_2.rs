use std::fmt;

// Custom assert macros
macro_rules! assert {
    ($cond:expr) => {
        if !$cond {
            panic!("assertion failed: {}", stringify!($cond));
        }
    };
}

macro_rules! assert_warning {
    ($cond:expr) => {
        if !$cond {
            eprintln!("assertion warning: {}", stringify!($cond));
        }
    };
}

macro_rules! assert_debug {
    ($cond:expr, $msg:expr, $($arg:tt)*) => {
        if !$cond {
            panic!(concat!("debug assertion failed: ", $msg), $($arg)*);
        }
    };
}

macro_rules! assert_error {
    ($cond:expr, $msg:expr) => {
        if !$cond {
            panic!("error assertion failed: {}", $msg);
        }
    };
}

macro_rules! assert_fatal {
    ($cond:expr, $msg:expr) => {
        if !$cond {
            panic!("fatal assertion failed: {}", $msg);
        }
    };
}

macro_rules! assert_custom {
    ($id:expr, $cond:expr, $msg:expr) => {
        if !$cond {
            panic!("custom assertion ({}): {}", $id, $msg);
        }
    };
}

fn trigger_assert_debug_even(i: i32) {
    assert_debug!((i % 2) == 0, "not an even number: {}", i);
}

fn trigger_assert_debug_odd(i: i32) {
    assert_debug!((i % 2) != 0, "not an odd number: {}", i);
}

fn trigger_assert_error() {
    let ptr: *const () = std::ptr::null();
    assert_error!(!ptr.is_null(), "invalid ptr: must not be null");
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
    assert_warning!(true);
    assert_debug!(true, "Debug message: {}", "No additional args");
    assert_error!(true, "Error message");
    assert_fatal!(true, "Fatal message");
    assert_custom!(0, true, "Custom message");

    for i in 0..5 {
        trigger_assert_debug_even(i);
    }

    for i in 0..5 {
        trigger_assert_debug_odd(i);
    }

    if let Err(e) = std::panic::catch_unwind(|| {
        trigger_assert_error();
    }) {
        if let Some(err) = e.downcast_ref::<String>() {
            println!("AssertionException caught:");
            println!("  [what]:       {}", err);
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

    println!();
    println!("if you see this message, this means you decided to ignore all assertions");
}