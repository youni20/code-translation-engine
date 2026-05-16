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

#![allow(dead_code)]

#[cfg(not(feature = "DEBUG_ASSERT_NO_STDIO"))]
use std::io::{self, Write};

// Hints for compilers about unreachable code and inline functions
macro_rules! DEBUG_ASSERT_MARK_UNREACHABLE {
    () => {
        std::hint::unreachable_unchecked()
    };
}

macro_rules! DEBUG_ASSERT_FORCE_INLINE {
    ($i:item) => {
        #[inline(always)]
        $i
    };
}

//=== source location ===//
/// Defines a location in the source code.
#[derive(Clone, Copy)]
struct SourceLocation {
    file_name: &'static str, // The file name.
    line_number: u32,        // The line number.
}

/// Expands to the current [SourceLocation].
macro_rules! DEBUG_ASSERT_CUR_SOURCE_LOCATION {
    () => {
        SourceLocation {
            file_name: file!(),
            line_number: line!() as u32,
        }
    };
}

//=== level ===//
/// Tag type to indicate the level of an assertion.
struct Level<const LEVEL: u32>;

/// Helper class that sets a certain level.
/// Inherit from it in your module handler.
struct SetLevel<const LEVEL: u32>;

impl<const LEVEL: u32> SetLevel<LEVEL> {
    const LEVEL: u32 = LEVEL;
}

/// Helper class that controls whether the handler can throw or not.
struct AllowException;

impl AllowException {
    const THROWING_EXCEPTION_IS_ALLOWED: bool = true;
}

//=== handler ===//
/// Does not do anything to handle a failed assertion (except calling process::abort()).
/// Inherit from it in your module handler.
struct NoHandler;

impl NoHandler {
    fn handle(_: SourceLocation, _: &str, _: Option<&str>) {}
}

/// The default handler that writes a message to `stderr`.
/// Inherit from it in your module handler.
struct DefaultHandler;

impl DefaultHandler {
    /// Prints a message to `stderr`.
    /// If `DEBUG_ASSERT_NO_STDIO` is defined, it will do nothing.
    #[inline]
    fn handle(loc: SourceLocation, expression: &str, message: Option<&str>) {
        #[cfg(not(feature = "DEBUG_ASSERT_NO_STDIO"))]
        {
            let _ = if expression.is_empty() {
                if let Some(msg) = message {
                    writeln!(
                        io::stderr(),
                        "[debug assert] {}: Unreachable code reached - {}.",
                        loc.file_name, msg
                    )
                } else {
                    writeln!(
                        io::stderr(),
                        "[debug assert] {}: Unreachable code reached.",
                        loc.file_name
                    )
                }
            } else if let Some(msg) = message {
                writeln!(
                    io::stderr(),
                    "[debug assert] {}: Assertion '{}' failed - {}.",
                    loc.file_name, expression, msg
                )
            } else {
                writeln!(
                    io::stderr(),
                    "[debug assert] {}: Assertion '{}' failed.",
                    loc.file_name, expression
                )
            };
        }
    }
}

/// \exclude
mod detail {
    use super::*;
    use std::process;
    use std::fmt::Debug;

    //=== boilerplate ===//

    // from http://en.cppreference.com/w/cpp/types/remove_reference
    trait RemoveReference {
        type Type;
    }

    impl<T> RemoveReference for T {
        type Type = T;
    }

    //=== helper class to check if throw is allowed ===//
    pub trait AllowsException {
        const VALUE: bool;
    }

    impl AllowsException for NoHandler {
        const VALUE: bool = false;
    }

    impl AllowsException for AllowException {
        const VALUE: bool = AllowException::THROWING_EXCEPTION_IS_ALLOWED;
    }

    //=== regular void fake ===//
    pub struct RegularVoid;

    impl RegularVoid {
        pub const fn new() -> Self {
            RegularVoid
        }
    }

    //=== assert implementation ===//
    pub fn debug_assertion_failed<Handler, Args: Clone + Debug>(
        _loc: SourceLocation,
        _expression: &str,
        _args: Args,
    ) -> RegularVoid
    where
        Handler: AllowsException,
    {
        #[allow(unreachable_code)]
        {
            process::abort();
            RegularVoid::new()
        }
    }

    DEBUG_ASSERT_FORCE_INLINE! {
        pub fn do_assert<Expr, Handler, Args: Clone + Debug>(
            expr: Expr,
            loc: SourceLocation,
            expression: &str,
            _handler: Handler,
            args: Args,
        ) -> RegularVoid
        where
            Expr: FnOnce() -> bool,
            Handler: AllowsException,
        {
            if expr() {
                RegularVoid::new()
            } else {
                debug_assertion_failed::<Handler, Args>(loc, expression, args)
            }
        }
    }

    pub const fn always_false() -> bool {
        false
    }
}

//=== assertion macros ===//

#[cfg(not(feature = "DEBUG_ASSERT_DISABLE"))]
macro_rules! DEBUG_ASSERT {
    ($expr:expr, $handler:expr, $($args:tt)*) => {
        detail::do_assert(
            || $expr,
            DEBUG_ASSERT_CUR_SOURCE_LOCATION!(),
            stringify!($expr),
            $handler,
            ($($args)*),
        )
    };
}

#[cfg(feature = "DEBUG_ASSERT_DISABLE")]
macro_rules! DEBUG_ASSERT {
    ($expr:expr, $handler:expr, $($args:tt)*) => {};
}

#[cfg(not(feature = "DEBUG_ASSERT_DISABLE"))]
macro_rules! DEBUG_UNREACHABLE {
    ($handler:expr, $($args:tt)*) => {
        detail::do_assert(
            detail::always_false,
            DEBUG_ASSERT_CUR_SOURCE_LOCATION!(),
            "",
            $handler,
            ($($args)*),
        )
    };
}

#[cfg(feature = "DEBUG_ASSERT_DISABLE")]
macro_rules! DEBUG_UNREACHABLE {
    ($handler:expr, $($args:tt)*) => {
        DEBUG_ASSERT_MARK_UNREACHABLE!()
    };
}

fn main() {
    // Example assertion
    DEBUG_ASSERT!(1 + 1 == 2, NoHandler, ());
    DEBUG_UNREACHABLE!(NoHandler, ());
}