#![allow(unused_variables)]
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};

const PPK_ASSERT_EXCEPTION_MESSAGE_BUFFER_SIZE: usize = 1024;

#[derive(Debug)]
pub struct AssertionException {
    file: &'static str,
    line: u32,
    function: &'static str,
    expression: &'static str,
    message: Option<String>,
}

impl AssertionException {
    pub fn new(file: &'static str, line: u32, function: &'static str, expression: &'static str, message: Option<String>) -> Self {
        AssertionException {
            file,
            line,
            function,
            expression,
            message,
        }
    }
}

impl Display for AssertionException {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "Assertion failed at {}:{}: {}\nExpression: {}\nMessage: {:?}",
            self.file, self.line, self.function, self.expression, self.message
        )
    }
}

impl Error for AssertionException {}

pub mod ppk {
    pub mod assert {
        pub mod implementation {
            use std::sync::atomic::{AtomicBool, Ordering};
            use std::process;

            #[derive(Debug)]
            pub enum AssertLevel {
                Warning,
                Debug,
                Error,
                Fatal,
            }

            #[derive(Debug)]
            pub enum AssertAction {
                None,
                Abort,
                Break,
                Ignore,
                IgnoreAll,
                Throw,
            }

            pub type AssertHandler = fn(&'static str, u32, &'static str, &'static str, AssertLevel, Option<&mut bool>, Option<String>) -> AssertAction;

            static IGNORE_ALL_ASSERTS: AtomicBool = AtomicBool::new(false);

            pub fn handle_assert(file: &'static str, line: u32, function: &'static str, expression: &'static str, level: AssertLevel, ignore_line: Option<&mut bool>, message: Option<String>) -> AssertAction {
                if IGNORE_ALL_ASSERTS.load(Ordering::SeqCst) {
                    return AssertAction::IgnoreAll;
                }

                match level {
                    AssertLevel::Fatal => {
                        eprintln!("Fatal assertion failed in {} at {}:{}: {}\nMessage: {:?}", function, file, line, expression, message);
                        process::abort();
                    },
                    _ => {
                        eprintln!("Assertion failed in {} at {}:{}: {}\nMessage: {:?}", function, file, line, expression, message);
                        AssertAction::Break
                    }
                }
            }

            pub fn set_assert_handler(handler: AssertHandler) -> AssertHandler {
                unimplemented!()
            }

            pub fn ignore_all_asserts(value: bool) {
                IGNORE_ALL_ASSERTS.store(value, Ordering::SeqCst);
            }

            pub fn ignore_all_asserts_value() -> bool {
                IGNORE_ALL_ASSERTS.load(Ordering::SeqCst)
            }

            static mut ASSERT_HANDLER: Option<AssertHandler> = None;

            pub fn get_handler() -> Option<AssertHandler> {
                unsafe { ASSERT_HANDLER }
            }

            #[macro_export]
            macro_rules! ppk_assert {
                ($level:expr, $expression:expr $(,$message:expr)?) => {
                    if !($expression) {
                        let message = $crate::ppk::assert::implementation::construct_message($($message)?);
                        if let Some(handler) = $crate::ppk::assert::implementation::get_handler() {
                            let action = handler(file!(), line!(), module_path!(), stringify!($expression), $level, None, message.clone());

                            if let ppk::assert::implementation::AssertAction::Break = action {
                                ppk::assert::implementation::debug_break();
                            }
                        } else {
                            ppk::assert::implementation::handle_assert(file!(), line!(), module_path!(), stringify!($expression), $level, None, message);
                        }
                    }
                };
            }

            pub fn debug_break() {
                if cfg!(target_os = "windows") {
                    // Equivalent to __debugbreak() in MSVC
                    extern "C" {
                        fn DebugBreak();
                    }
                    unsafe { DebugBreak() }
                } else if cfg!(target_os = "linux") || cfg!(target_os = "macos") {
                    // Directly invoke a breakpoint instruction
                    unsafe { std::arch::asm!("int3"); }
                } else {
                    unreachable!("Unsupported platform");
                }
            }

            pub fn construct_message(message: Option<impl ToString>) -> Option<String> {
                message.map(|m| m.to_string())
            }

        }
    }
}

fn main() {
    ppk_assert!(ppk::assert::implementation::AssertLevel::Debug, 2 + 2 == 4, Some("Checking math"));
}