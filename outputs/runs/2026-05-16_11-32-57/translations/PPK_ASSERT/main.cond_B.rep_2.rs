use std::vec::Vec;

fn assert(condition: bool, message: &str) {
    if !condition {
        panic!("{}", message);
    }
}

fn trigger_assert_debug_even(i: i32) {
    assert((i % 2) == 0, &format!("not an even number: {}", i));
}

fn trigger_assert_debug_odd(i: i32) {
    assert((i % 2) != 0, &format!("not an odd number: {}", i));
}

fn trigger_assert_error() {
    let ptr: *const () = std::ptr::null();
    assert(!ptr.is_null(), "invalid ptr: must not be null");
}

fn trigger_assert_custom1() {
    let ptr: *const () = std::ptr::null();
    assert!(!ptr.is_null(), "invalid ptr: must not be null");
    // Note: The custom assertion ID (100) is omitted since Rust's assert! does not support custom IDs.
}

fn trigger_assert_custom2() {
    let ptr: *const () = std::ptr::null();
    assert!(!ptr.is_null(), "invalid ptr: must not be null");
}

fn trigger_assert_unused_return_value1() -> Vec<i32> {
    vec![0; 10]
}

fn trigger_assert_unused_return_value2() -> i32 {
    0
}

fn main() {
    assert(true, ""); // Normal assert 

    for i in 0..5 {
        trigger_assert_debug_even(i);
    }

    for i in 0..5 {
        trigger_assert_debug_odd(i);
    }

    if let Err(e) = std::panic::catch_unwind(|| trigger_assert_error()) {
        println!("AssertionException caught:");
        // Panic payload is usually a String or &str in Rust
        if let Some(s) = e.downcast_ref::<&str>() {
            println!("  [what]: {}", s);
        } else {
            println!("  [what]: unknown panic payload");
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

    assert(false, "He's dead. He's dead, Jim");

    println!("\nif you see this message, this means you decided to ignore all assertions");
}