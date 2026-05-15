//======================================================================//
// This software is provided 'as-is', without any express or
// implied warranty. In no event will the authors be held
// liable for any damages arising from the use of this software.
//
// Permission is granted to anyone to use this software for any purpose,
// including commercial applications, and to alter it and redistribute
// it freely, subject to the following restrictions:
//
// 1. The origin of this software must not be misrepresented;
//    you must not claim that you wrote the original software.
//    If you use this software in a product, an acknowledgment
//    in the product documentation would be appreciated but
//    is not required.
//
// 2. Altered source versions must be plainly marked as such,
//    and must not be misrepresented as being the original software.
//
// 3. This notice may not be removed or altered from any
//    source distribution.
//======================================================================//

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
macro_rules! unreachable {
    () => {};
}

/// Strong hint to the compiler to inline a function.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
macro_rules! inline {
    () => {
        #[inline(always)]
    };
}

pub mod debug_assert {
    //=== source location ===//
    /// Defines a location in the source code.
    #[derive(Copy, Clone)]
    pub struct SourceLocation {
        file_name: &'static str,
        line_number: u32,
    }

    /// Expands to the current [debug_assert::SourceLocation].
    #[macro_export]
    macro_rules! debug_assert_cur_source_location {
        () => {
            debug_assert::SourceLocation { 
                file_name: file!(), 
                line_number: line!()
            }
        };
    }

    //=== level ===//
    /// Tag type to indicate the level of an assertion.
    pub struct Level<const LEVEL: u32>;

    /// Helper class that sets a certain level.
    /// Inherit from it in your module handler.
    pub trait SetLevel {
        const LEVEL: u32;
    }

    /// Helper class that controls whether the handler can throw or not.
    /// Inherit from it in your module handler.
    /// If the module does not implement this trait, it is assumed that
    /// the handle does not throw.
    pub trait AllowException {
        const THROWING_EXCEPTION_IS_ALLOWED: bool = true;
    }

    //=== handler ===//
    /// Does not do anything to handle a failed assertion (except calling
    /// [std::abort()]).
    /// Inherit from it in your module handler.
    pub trait NoHandler {
        /// \effects Does nothing.
        /// \notes Can take any additional arguments.
        fn handle<'a, Args>(_loc: &SourceLocation, _expression: &str, _args: Args)
        where
            Args: std::fmt::Debug
        {
        }
    }

    /// The default handler that writes a message to `stderr`.
    /// Inherit from it in your module handler.
    pub struct DefaultHandler;

    impl DefaultHandler {
        /// \effects Prints a message to `stderr`.
        /// \notes It can optionally accept an additional message string.
        #[allow(unused_variables)]
        pub fn handle(loc: &SourceLocation, expression: &str, message: Option<&str>) {
            {
                let msg = if expression.is_empty() {
                    if let Some(message) = message {
                        format!(
                            "[debug assert] {}: Unreachable code reached - {}.",
                            loc.file_name, message
                        )
                    } else {
                        format!(
                            "[debug assert] {}: Unreachable code reached.",
                            loc.file_name
                        )
                    }
                } else if let Some(message) = message {
                    format!(
                        "[debug assert] {}: Assertion '{}' failed - {}.",
                        loc.file_name, expression, message
                    )
                } else {
                    format!("[debug assert] {}: Assertion '{}' failed.", loc.file_name, expression)
                };
                eprintln!("{}", msg);
            }
        }
    }

    pub mod detail {
        use crate::debug_assert::{NoHandler, SetLevel};

        //=== boilerplate ===//
        pub struct RemoveReference<T>(T);

        impl<T> RemoveReference<T> {
            pub fn forward(t: T) -> T {
                t
            }
        }

        pub struct EnableIf<const VALUE: bool>;

        pub struct EnableIfTrue;

        impl EnableIf<true> {
            // Removed the associated type to fix the error
        }

        ///=== assert implementation ===//
        /// function name will be shown on const assertion failure
        pub fn debug_assertion_failed<H, Args>(
            loc: &super::SourceLocation,
            expression: &str,
            handler: H,
            args: Args,
        ) -> !
        where
            H: NoHandler,
            Args: std::fmt::Debug,
        {
            H::handle(loc, expression, args);
            std::process::abort();
        }

        pub fn do_assert<F, H, const L: u32, Args>(
            expr: F,
            loc: &super::SourceLocation,
            expression: &str,
            _handler: H,
            _level: &super::Level<L>,
            args: Args,
        ) where
            F: FnOnce() -> bool,
            H: NoHandler + SetLevel,
            Args: std::fmt::Debug,
        {
            assert!(L > 0, "level of an assertion must not be 0");
            if L <= H::LEVEL {
                if !expr() {
                    debug_assertion_failed(loc, expression, _handler, args);
                }
            }
        }

        pub fn always_false() -> bool {
            false
        }
    }
}

//=== assertion macros ===//
#[macro_export]
macro_rules! debug_assert {
    ($expr:expr, $handler:expr) => {
        debug_assert!($expr, $handler, debug_assert::Level::<1>, ())
    };
    ($expr:expr, $handler:expr, $level:expr) => {
        debug_assert!($expr, $handler, $level, ())
    };
    ($expr:expr, $handler:expr, $level:expr, $($args:tt)*) => {
        {
            $crate::debug_assert::detail::do_assert(
                || $expr,
                &$crate::debug_assert_cur_source_location!(),
                stringify!($expr),
                &$handler,
                &$level,
                ($($args)*)
            );
        }
    };
}

#[macro_export]
macro_rules! debug_unreachable {
    ($handler:expr) => {
        debug_unreachable!($handler, debug_assert::Level::<1>)
    };
    ($handler:expr, $level:expr) => {
        debug_unreachable!($handler, $level, ())
    };
    ($handler:expr, $level:expr, $($args:tt)*) => {
        {
            $crate::debug_assert::detail::do_assert(
                || $crate::debug_assert::detail::always_false(),
                &$crate::debug_assert_cur_source_location!(),
                "",
                &$handler,
                &$level,
                ($($args)*)
            );
        }
    };
}

fn main() {
    // Add main function to compile the program successfully.
}