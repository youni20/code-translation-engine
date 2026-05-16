// see README.md for usage instructions.
// (‑●‑●)> released under the WTFPL v2 license, by Gregory Pakosz (@gpakosz)

// -- usage --------------------------------------------------------------------
// This Rust implementation translates concepts of assertions from C++ to Rust.

use std::fmt;

#[cfg(debug_assertions)]
const PPK_ASSERT_ENABLED: bool = true;

#[cfg(not(debug_assertions))]
const PPK_ASSERT_ENABLED: bool = false;

#[derive(Debug, Clone, Copy)]
enum AssertLevel {
    Warning,
    Debug,
    Error,
    Fatal,
}

impl Default for AssertLevel {
    fn default() -> Self {
        AssertLevel::Debug
    }
}

#[derive(Debug)]
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
        message: Option<String>,
    ) -> Self {
        AssertionException {
            file,
            line,
            function,
            expression,
            message,
        }
    }
}

impl fmt::Display for AssertionException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Assertion failed in file {}, line {}, function {}: {}",
            self.file, self.line, self.function, self.expression
        )?;
        if let Some(ref msg) = self.message {
            write!(f, ", message: {}", msg)?;
        }
        Ok(())
    }
}

impl std::error::Error for AssertionException {}

fn handle_assert<F>(
    file: &'static str,
    line: u32,
    function: &'static str,
    expression: &'static str,
    _level: AssertLevel,
    message: Option<String>,
    action_fn: F,
) where
    F: FnOnce() -> AssertAction,
{
    if PPK_ASSERT_ENABLED {
        if let AssertAction::Break = action_fn() {
            panic!(
                "{}",
                AssertionException::new(file, line, function, expression, message)
            );
        }
    }
}

#[derive(Debug)]
enum AssertAction {
    None,
    Abort,
    Break,
    Ignore,
}

macro_rules! ppk_assert {
    ($level:expr, $expression:expr $(, $($message:tt)*)?) => {
        if cfg!(debug_assertions) {
            if !$expression {
                handle_assert(
                    file!(),
                    line!(),
                    module_path!(),
                    stringify!($expression),
                    $level,
                    Some(format!($($($message)*)?).to_string()).filter(|s| !s.is_empty()),
                    || AssertAction::Break,
                );
            }
        }
    };
}

macro_rules! ppk_static_assert {
    ($expression:expr $(, $message:expr)?) => {
        const _: bool = $expression;
    };
}

// Usage Example
fn main() {
    ppk_assert!(AssertLevel::Debug, true);
    ppk_static_assert!(true);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_assert() {
        ppk_assert!(AssertLevel::Debug, true);
    }

    #[test]
    fn test_static_assert() {
        ppk_static_assert!(true);
    }
}