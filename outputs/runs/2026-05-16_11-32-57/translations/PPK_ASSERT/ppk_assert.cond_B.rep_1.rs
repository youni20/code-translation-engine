// see README.md for usage instructions.
// (‑●‑●)> released under the WTFPL v2 license, by Gregory Pakosz (@gpakosz)

// -- usage --------------------------------------------------------------------

/*

  run-time assertions:

    PPK_ASSERT(expression);
    PPK_ASSERT(expression, message, ...);

    PPK_ASSERT_WARNING(expression);
    PPK_ASSERT_WARNING(expression, message, ...);

    PPK_ASSERT_DEBUG(expression);
    PPK_ASSERT_DEBUG(expression, message, ...);

    PPK_ASSERT_ERROR(expression);
    PPK_ASSERT_ERROR(expression, message);

    PPK_ASSERT_FATAL(expression);
    PPK_ASSERT_FATAL(expression, message, ...);

    PPK_ASSERT_CUSTOM(level, expression);
    PPK_ASSERT_CUSTOM(level, expression, message, ...);

    PPK_ASSERT_USED(type)
    PPK_ASSERT_USED_WARNING(type)
    PPK_ASSERT_USED_DEBUG(type)
    PPK_ASSERT_USED_ERROR(type)
    PPK_ASSERT_USED_FATAL(type)
    PPK_ASSERT_USED_CUSTOM(level, type)

    PPK_ASSERT_USED(bool) foo()
    {
      return true;
    }

  compile-time assertions:

    PPK_STATIC_ASSERT(expression)
    PPK_STATIC_ASSERT(expression, message)

*/

#[cfg(debug_assertions)]
mod ppk_assert {
    pub const PPK_ASSERT_ENABLED: bool = true;
}

#[cfg(not(debug_assertions))]
mod ppk_assert {
    pub const PPK_ASSERT_ENABLED: bool = false;
}

pub mod ppk {
    pub mod assert {
        pub mod implementation {
            pub mod AssertLevel {
                pub const WARNING: i32 = 32;
                pub const DEBUG: i32 = 64;
                pub const ERROR: i32 = 128;
                pub const FATAL: i32 = 256;

                pub const PPK_ASSERT_DEFAULT_LEVEL: i32 = DEBUG; // Change if needed
            }

            pub mod AssertAction {
                pub enum AssertAction {
                    None,
                    Abort,
                    Break,
                    Ignore,
                    IgnoreLine,
                    IgnoreAll,
                    Throw,
                }
            }

            #[cfg(target_os = "windows")]
            fn debug_break() {
                unsafe { winapi::um::debugapi::DebugBreak(); }
            }

            #[cfg(not(target_os = "windows"))]
            fn debug_break() {
                #[cfg(any(target_os = "linux", target_os = "android", target_os = "macos"))]
                {
                    extern "C" {
                        fn raise(sig: i32) -> i32;
                    }
                    unsafe { raise(5); } // 5 corresponds to SIGTRAP on POSIX systems
                }
            }

            pub fn handle_assert(
                file: &str,
                line: u32,
                function: &str,
                expression: &str,
                level: i32,
                message: Option<&str>,
            ) -> AssertAction::AssertAction {
                eprintln!(
                    "Assertion failed at {}: {} in {}: `{}`. Level: {}, Message: {}",
                    file, line, function, expression, level, message.unwrap_or("")
                );
                AssertAction::AssertAction::Break
            }

            #[macro_export]
            macro_rules! PPK_ASSERT {
                ($level:expr, $expression:expr) => {
                    if !$expression {
                        ppk::assert::implementation::debug_break();
                    }
                };
                ($level:expr, $expression:expr, $message:expr) => {
                    if !$expression {
                        let _ = ppk::assert::implementation::handle_assert(
                            file!(),
                            line!(),
                            module_path!(),
                            stringify!($expression),
                            $level,
                            Some($message),
                        );
                        ppk::assert::implementation::debug_break();
                    }
                };
            }

            pub fn ignore_all_asserts(_value: bool) {
                // Stub function to mimic behavior
            }

            pub fn ignore_all_asserts_active() -> bool {
                false // Stub implementation
            }
        }
    }
}

fn main() {
    // Main function to satisfy the Rust compiler
}