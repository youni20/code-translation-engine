// See README.md for usage instructions.
// (‑●‑●)> released under the WTFPL v2 license, by Gregory Pakosz (@gpakosz)

// Necessary imports
use std::fmt;

// Default assertion levels and settings
const PPK_ASSERT_ENABLED: bool = !cfg!(nDEBUG);
const PPK_ASSERT_DEFAULT_LEVEL: AssertLevel = AssertLevel::Debug;

// Helper macro to concatenate tokens
macro_rules! ppk_assert_join {
    ($lhs:ident, $rhs:ident) => {
        ppk_assert_join_impl!($lhs, $rhs)
    };
    ($lhs:expr, $rhs:ident) => {
        ppk_assert_join_impl!($lhs, $rhs)
    };
}

macro_rules! ppk_assert_join_impl {
    ($lhs:expr, $rhs:ident) => {
        concat!($lhs, stringify!($rhs))
    };
}

// Function signature and file/line/function constants
const PPK_ASSERT_FILE: &str = file!();
macro_rules! ppk_assert_line {
    () => {
        line!()
    };
}
macro_rules! ppk_assert_function {
    () => {
        if cfg!(any(gcc, clang)) {
            std::any::type_name::<fn()>()
        } else {
            "unknown_function"
        }
    };
}

// Assertion levels as enum
#[derive(Debug, PartialEq, PartialOrd)]
enum AssertLevel {
    Warning,
    Debug,
    Error,
    Fatal,
}

// Assertion actions as enum
#[derive(Debug, PartialEq)]
enum AssertAction {
    None,
    Abort,
    Break,
    Ignore,
    #[cfg(not(feature = "disable_ignore_line"))]
    IgnoreLine,
    IgnoreAll,
    Throw,
}

// Exception structure
#[derive(Debug, Clone)]
struct AssertionException {
    file: &'static str,
    line: u32,
    function: &'static str,
    expression: &'static str,
    message: Option<String>,
}

impl AssertionException {
    fn new(
        file: &'static str, 
        line: u32, 
        function: &'static str, 
        expression: &'static str, 
        message: Option<String>
    ) -> Self {
        Self {
            file,
            line,
            function,
            expression,
            message,
        }
    }
}

impl fmt::Display for AssertionException {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref msg) = self.message {
            write!(
                f,
                "Assertion failed at {}: {} in function {}: {}\nMessage: {}",
                self.file, self.line, self.function, self.expression, msg
            )
        } else {
            write!(
                f,
                "Assertion failed at {}: {} in function {}: {}",
                self.file, self.line, self.function, self.expression
            )
        }
    }
}

// Assert handlers
type AssertHandler = fn(&'static str, u32, &'static str, &'static str, AssertLevel, Option<String>) -> AssertAction;

fn handle_assert(file: &'static str, line: u32, function: &'static str, expression: &'static str, level: AssertLevel, message: Option<String>) -> AssertAction {
    if level >= AssertLevel::Error {
        println!("Assertion Error: {}", AssertionException::new(file, line, function, expression, message.clone()));
        AssertAction::Break
    } else {
        AssertAction::Ignore
    }
}

fn set_assert_handler(_handler: AssertHandler) -> AssertHandler {
    // This function is for demonstration purposes only.
    handle_assert
}

fn ignore_all_asserts(_: bool) {
    // Ignoring all assertions (not implemented)
}

// Helper macros for assertions
macro_rules! PPK_ASSERT {
    ($level:expr, $expression:expr $(, $msg:expr)?) => {
        if !$expression {
            let msg: Option<String> = None$(.or(Some($msg.to_string())))?;
            match handle_assert(PPK_ASSERT_FILE, ppk_assert_line!(), ppk_assert_function!(), stringify!($expression), $level, msg) {
                AssertAction::Break => panic!("Break: Assertion failed"),
                _ => (), // other actions
            }
        }
    };
}

// Static assertion based on const-eval context
macro_rules! PPK_STATIC_ASSERT {
    ($expression:expr $(, $message:expr)?) => {
        const _: () = {
            if !$expression {
                panic!("{}", if let Some(msg) = $message { msg } else { "Static assertion failed" });
            }
        };
    };
}

// Assertion example usage
fn foo() -> bool {
    PPK_ASSERT!(AssertLevel::Debug, 2 + 2 == 4);
    true
}

fn main() {
    foo();
}