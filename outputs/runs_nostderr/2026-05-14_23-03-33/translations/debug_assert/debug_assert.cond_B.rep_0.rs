//======================================================================//
// Copyright (C) 2016-2018 Jonathan Müller <jonathanmueller.dev@gmail.com>
//
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

#[cfg(not(feature = "disable_debug_assert"))]
mod debug_assert {
    use std::fmt;

    //=== source location ===//
    /// Defines a location in the source code.
    pub struct SourceLocation {
        file_name: &'static str,
        line_number: u32,
    } ///< The file name. The line number.

    /// Expands to the current [debug_assert::SourceLocation].
    #[macro_export]
    macro_rules! DEBUG_ASSERT_CUR_SOURCE_LOCATION {
        () => {
            debug_assert::SourceLocation {
                file_name: file!(),
                line_number: line!(),
            }
        };
    }

    //=== level ===//
    /// Tag type to indicate the level of an assertion.
    pub struct Level<const LEVEL: u32>;

    /// Helper class that sets a certain level.
    /// Inherit from it in your module handler.
    pub struct SetLevel<const LEVEL: u32>;

    impl<const LEVEL: u32> SetLevel<LEVEL> {
        pub const LEVEL: u32 = LEVEL;
    }

    /// Helper class that controls whether the handler can throw or not.
    pub struct AllowException;

    impl AllowException {
        pub const THROWING_EXCEPTION_IS_ALLOWED: bool = true;
    }

    //=== handler ===//
    pub struct NoHandler;

    impl NoHandler {
        /// \effects Does nothing.
        /// \notes Can take any additional arguments.
        pub fn handle(_: &SourceLocation, _: &str, _: &[&dyn fmt::Debug]) {}
    }

    /// The default handler that writes a message to `stderr`.
    /// Inherit from it in your module handler.
    pub struct DefaultHandler;

    impl DefaultHandler {
        /// \effects Prints a message to `stderr`.
        /// \notes It can optionally accept an additional message string.
        pub fn handle(loc: &SourceLocation, expression: &str, args: &[&dyn fmt::Debug]) {
            if expression.is_empty() {
                if !args.is_empty() {
                    eprintln!(
                        "[debug assert] {}: {}: Unreachable code reached - {:?}.",
                        loc.file_name, loc.line_number, args[0]
                    );
                } else {
                    eprintln!(
                        "[debug assert] {}: {}: Unreachable code reached.",
                        loc.file_name, loc.line_number
                    );
                }
            } else if !args.is_empty() {
                eprintln!(
                    "[debug assert] {}: {}: Assertion '{}' failed - {:?}.",
                    loc.file_name, loc.line_number, expression, args[0]
                );
            } else {
                eprintln!(
                    "[debug assert] {}: {}: Assertion '{}' failed.",
                    loc.file_name, loc.line_number, expression
                );
            }
        }
    }

    /// \exclude
    mod detail {
        use super::SourceLocation;
        use std::fmt;
        use std::process;

        // === regular void fake ===//
        pub struct RegularVoid;

        impl Default for RegularVoid {
            fn default() -> Self {
                Self
            }
        }

        //=== assert implementation ===//
        pub fn debug_assertion_failed<Handler: Fn(&SourceLocation, &str, &[&dyn fmt::Debug])>(
            handler: Handler,
            loc: &SourceLocation,
            expression: &str,
            args: &[&dyn fmt::Debug],
        ) -> RegularVoid {
            handler(loc, expression, args);
            process::abort()
        }

        pub fn do_assert<Expr, Handler, const LEVEL: u32>(
            expr: Expr,
            loc: &SourceLocation,
            expression: &str,
            handler: Handler,
            args: &[&dyn fmt::Debug],
        ) -> Result<RegularVoid, RegularVoid>
        where
            Expr: Fn() -> bool,
            Handler: Fn(&SourceLocation, &str, &[&dyn fmt::Debug]),
        {
            if expr() {
                Ok(RegularVoid)
            } else {
                Err(debug_assertion_failed(handler, loc, expression, args))
            }
        }

        pub fn always_false() -> bool {
            false
        }
    }
}

/// The assertion macro.
#[macro_export]
macro_rules! DEBUG_ASSERT {
    ($expr:expr, $handler:expr $(, $arg:expr)*) => {
        let args: &[&dyn std::fmt::Debug] = &[$(&$arg,)*];
        if let Err(_) = debug_assert::detail::do_assert(
            || { $expr },
            &DEBUG_ASSERT_CUR_SOURCE_LOCATION!(),
            stringify!($expr),
            $handler,
            args,
        ) {
            std::process::abort();
        }
    };
}

/// Marks a branch as unreachable.
#[macro_export]
macro_rules! DEBUG_UNREACHABLE {
    ($handler:expr $(, $arg:expr)*) => {
        let args: &[&dyn std::fmt::Debug] = &[$(&$arg,)*];
        if let Err(_) = debug_assert::detail::do_assert(
            || { debug_assert::detail::always_false() },
            &DEBUG_ASSERT_CUR_SOURCE_LOCATION!(),
            "",
            $handler,
            args,
        ) {
            std::process::abort();
        }
    };
}

#[cfg(feature = "disable_debug_assert")]
#[macro_export]
macro_rules! DEBUG_ASSERT {
    ($expr:expr, $handler:expr $(, $arg:expr)*) => {};
}

#[cfg(feature = "disable_debug_assert")]
#[macro_export]
macro_rules! DEBUG_UNREACHABLE {
    ($handler:expr $(, $arg:expr)*) => {
        compiler_error!("Unreachable code has been reached")
    };
}

fn main() {
    // Main function to satisfy compiler requirement
}