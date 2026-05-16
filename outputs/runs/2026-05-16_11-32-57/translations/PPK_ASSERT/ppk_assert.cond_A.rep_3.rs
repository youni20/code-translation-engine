// see README.md for usage instructions.
// (‑●‑●)> released under the WTFPL v2 license, by Gregory Pakosz (@gpakosz)

// -- usage --------------------------------------------------------------------
// This section is not implemented in Rust since macros cannot be explained directly in comments.

use std::fmt;

#[cfg(debug_assertions)]
const PPK_ASSERT_ENABLED: bool = true;

#[derive(Debug)]
enum AssertLevel {
    Warning = 32,
    Debug = 64,
    Error = 128,
    Fatal = 256,
}

#[derive(Debug, PartialEq)]
enum AssertAction {
    None,
    Abort,
    Break,
    Ignore,
    IgnoreLine,
    IgnoreAll,
    Throw,
}

#[allow(dead_code)]
fn handle_assert(
    file: &str,
    line: u32,
    function: &str,
    expression: &str,
    level: AssertLevel,
    message: Option<&str>,
) -> AssertAction {
    println!(
        "Assertion failed: file: {}, line: {}, function: {}, expression: {}, level: {:?}, message: {:?}",
        file, line, function, expression, level, message
    );
    AssertAction::Break // As an example, you could set this to any AssertAction
}

macro_rules! ppk_assert {
    ($level:expr, $expression:expr, $($message:expr),*) => {{
        if !$expression && !ignore_all_asserts() {
            if handle_assert(
                file!(),
                line!(),
                std::any::type_name::<fn()>(),
                stringify!($expression),
                $level,
                Some(&format!($($message),*)),
            ) == AssertAction::Break {
                ppk_assert_debug_break();
            }
        }
    }};
    ($level:expr, $expression:expr) => {{
        if !$expression && !ignore_all_asserts() {
            if handle_assert(
                file!(),
                line!(),
                std::any::type_name::<fn()>(),
                stringify!($expression),
                $level,
                None,
            ) == AssertAction::Break {
                ppk_assert_debug_break();
            }
        }
    }};
}

fn ppk_assert_debug_break() {
    #[cfg(target_os = "windows")]
    unsafe {
        // On Windows
        std::arch::asm!("int3");
    }
    
    #[cfg(not(target_os = "windows"))]
    unsafe {
        // On Unix-like systems
        std::arch::asm!("int $3");
    }
}

static mut IGNORE_ALL_ASSERTS: bool = false;

fn set_ignore_all_asserts(value: bool) {
    unsafe { IGNORE_ALL_ASSERTS = value }
}

fn ignore_all_asserts() -> bool {
    unsafe { IGNORE_ALL_ASSERTS }
}

#[derive(Debug)]
pub struct AssertionException<'a> {
    file: &'a str,
    line: u32,
    function: &'a str,
    expression: &'a str,
    message: String,
}

impl std::fmt::Display for AssertionException<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Assertion failed at file: {}, line: {}, function: {}, expression: {}, message: {}",
               self.file,
               self.line,
               self.function,
               self.expression,
               self.message)
    }
}

impl std::error::Error for AssertionException<'_> {}

impl<'a> AssertionException<'a> {
    pub fn new(file: &'a str, line: u32, function: &'a str, expression: &'a str, message: &str) -> Self {
        Self {
            file,
            line,
            function,
            expression,
            message: message.to_string(),
        }
    }
}

fn main() {
    // Example usage of assertions
    let condition = false;

    ppk_assert!(AssertLevel::Debug, condition, "Example assertion failed");
}