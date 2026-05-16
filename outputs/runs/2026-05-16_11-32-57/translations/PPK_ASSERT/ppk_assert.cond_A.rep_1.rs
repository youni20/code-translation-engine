// see README.md for usage instructions.
// (‑●‑●)> released under the WTFPL v2 license, by Gregory Pakosz (@gpakosz)

use std::fmt;

// -- usage --------------------------------------------------------------------
// PPK_ASSERT_ENABLED and PPK_ASSERT_DEFAULT_LEVEL configuration
#[cfg(debug_assertions)]
const PPK_ASSERT_ENABLED: bool = true;

#[derive(Debug)]
enum AssertLevel {
    Warning,
    Debug,
    Error,
    Fatal,
    Custom(i32),
}

#[derive(Debug, Clone)]
struct AssertionException {
    file: &'static str,
    line: u32,
    function: &'static str,
    expression: &'static str,
    message: Option<String>,
}

impl fmt::Display for AssertionException {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Assertion failed: file '{}', line {}, function '{}' - expression '{}', message: {:?}",
            self.file, self.line, self.function, self.expression, self.message
        )
    }
}

// Assert Handlers
type AssertHandler = fn(&str, u32, &str, &str, AssertLevel, Option<&String>) -> AssertAction;

#[derive(Debug, Copy, Clone, PartialEq)]
enum AssertAction {
    None,
    Abort,
    Break,
    Ignore,
    IgnoreAll,
}

// Global state for assertion ignoring
static IGNORE_ALL_ASSERTS: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

fn set_ignore_all_asserts(value: bool) {
    IGNORE_ALL_ASSERTS.store(value, std::sync::atomic::Ordering::SeqCst);
}

fn get_ignore_all_asserts() -> bool {
    IGNORE_ALL_ASSERTS.load(std::sync::atomic::Ordering::SeqCst)
}

fn handle_assert(file: &str, line: u32, function: &str, expression: &str, level: AssertLevel, message: Option<&String>) -> AssertAction {
    // This is where actual assert handling logic would be implemented, like logging or breaking into a debugger.
    println!("Handling assert at file '{}', line {}, function '{}'. Expression: '{}', Level: {:?}, Message: {:?}", file, line, function, expression, level, message);
    
    if get_ignore_all_asserts() {
        AssertAction::IgnoreAll
    } else {
        AssertAction::Break
    }
}

// Assertion Macros
macro_rules! PPK_ASSERT {
    ($level:expr, $expression:expr) => {
        PPK_ASSERT!($level, $expression, None::<String>)
    };
    ($level:expr, $expression:expr, $message:expr) => {
        if $expression || get_ignore_all_asserts() {
            // No action needed
        } else {
            let action = handle_assert(file!(), line!(), std::module_path!(), stringify!($expression), $level, $message.as_ref());
            if action == AssertAction::Break {
                panic!("Breaking due to assertion failure.");
            }
        }
    };
}

macro_rules! PPK_STATIC_ASSERT {
    ($expression:expr) => {
        const _: () = assert!($expression, "Static assertion failed");
    };
    ($expression:expr, $message:expr) => {
        const _: () = assert!($expression, $message);
    };
}

fn main() {
    // Example usage of assertions
    PPK_ASSERT!(AssertLevel::Debug, 2 + 2 == 4);
    PPK_STATIC_ASSERT!(true, "This should always pass");
}