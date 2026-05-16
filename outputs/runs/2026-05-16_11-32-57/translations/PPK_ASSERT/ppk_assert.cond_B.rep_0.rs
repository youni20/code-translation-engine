#![allow(unused_macros)]

#[cfg(not(debug_assertions))]
const PPK_ASSERT_ENABLED: bool = false;

#[cfg(debug_assertions)]
const PPK_ASSERT_ENABLED: bool = true;

const PPK_ASSERT_DEFAULT_LEVEL: AssertLevel = AssertLevel::Debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AssertLevel {
    Warning = 32,
    Debug = 64,
    Error = 128,
    Fatal = 256,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AssertAction {
    None,
    Abort,
    Break,
    Ignore,
    #[cfg(not(PPK_ASSERT_DISABLE_IGNORE_LINE))]
    IgnoreLine,
    IgnoreAll,
    Throw,
}

type AssertHandler = fn(
    &str,
    u32,
    &str,
    &str,
    AssertLevel,
    Option<&str>,
) -> AssertAction;

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
        message: Option<&str>,
    ) -> Self {
        AssertionException {
            file,
            line,
            function,
            expression,
            message: message.map(|m| m.to_string()),
        }
    }

    fn what(&self) -> &str {
        self.message.as_deref().unwrap_or("Assertion Exception")
    }
}

#[inline(always)]
fn ppk_assert_likely(arg: bool) -> bool {
    arg
}

#[inline(always)]
fn ppk_assert_unlikely(arg: bool) -> bool {
    arg
}

#[macro_export]
macro_rules! ppk_assert {
    ($level:expr, $expr:expr) => {
        if ppk_assert_likely($expr) || ppk::assert::implementation::ignore_all_asserts() {
        } else {
            if ppk::assert::implementation::handle_assert(
                file!(),
                line!(),
                "",
                stringify!($expr),
                $level,
                None,
            ) == AssertAction::Break
            {
                ppk_assert_debug_break();
            }
        }
    };
    ($level:expr, $expr:expr, $msg:expr) => {
        if ppk_assert_likely($expr) || ppk::assert::implementation::ignore_all_asserts() {
        } else {
            if ppk::assert::implementation::handle_assert(
                file!(),
                line!(),
                "",
                stringify!($expr),
                $level,
                Some($msg),
            ) == AssertAction::Break
            {
                ppk_assert_debug_break();
            }
        }
    };
}

fn ppk_assert_debug_break() {
    #[cfg(target_os = "windows")]
    unsafe {
        asm!("int3");
    }

    #[cfg(not(target_os = "windows"))]
    {
        std::process::abort();
    }
}

#[allow(dead_code)]
mod ppk {
    pub mod assert {
        pub mod implementation {
            use super::super::super::{AssertAction, AssertLevel};

            static mut IGNORE_ALL_ASSERTS: bool = false;

            pub fn ignore_all_asserts() -> bool {
                unsafe { IGNORE_ALL_ASSERTS }
            }

            pub fn ignore_all_asserts_set(value: bool) {
                unsafe {
                    IGNORE_ALL_ASSERTS = value;
                }
            }

            pub fn handle_assert(
                file: &'static str,
                line: u32,
                function: &'static str,
                expression: &'static str,
                level: AssertLevel,
                message: Option<&str>,
            ) -> AssertAction {
                eprintln!(
                    "Assertion failed: File: {}, Line: {}, Function: {}, Expression: {}, Level: {:?}, Message: {}",
                    file,
                    line,
                    function,
                    expression,
                    level,
                    message.unwrap_or("No message")
                );
                AssertAction::Break
            }
        }
    }
}

fn main() {
    let condition = false;
    ppk_assert!(AssertLevel::Debug, condition, "This is a debug assertion!");
}