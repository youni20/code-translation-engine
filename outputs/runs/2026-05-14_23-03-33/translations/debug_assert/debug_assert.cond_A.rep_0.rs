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

use std::process::abort;

//=== source location ===//
/// Defines a location in the source code.
#[derive(Debug)]
struct SourceLocation {
    file_name: &'static str, // The file name.
    line_number: u32,        // The line number.
}

/// Expands to the current SourceLocation.
macro_rules! DEBUG_ASSERT_CUR_SOURCE_LOCATION {
    () => {
        SourceLocation { file_name: file!(), line_number: line!() }
    };
}

//=== level ===//
/// Tag type to indicate the level of an assertion.
struct Level<const LEVEL: u32>;

/// Helper class that sets a certain level.
/// Use this struct as a module handler.
struct SetLevel<const LEVEL: u32>;

/// Helper struct that controls whether the handler can throw or not.
/// Use this struct as a module handler.
/// If the module does not use this struct, it is assumed that the handler does not throw.
struct AllowException;

impl AllowException {
    const THROWING_EXCEPTION_IS_ALLOWED: bool = true;
}

//=== handler ===//
/// Does not do anything to handle a failed assertion (except calling abort()).
/// Use this struct as a module handler.
struct NoHandler;

impl NoHandler {
    /// \effects Does nothing.
    /// \notes Can take any additional arguments.
    fn handle(_location: &SourceLocation, _expression: &str, _args: Option<&str>) {}
}

/// The default handler that writes a message to `stderr`.
/// Use this struct as a module handler.
struct DefaultHandler;

impl DefaultHandler {
    /// \effects Prints a message to `stderr`.
    /// \notes It can optionally accept an additional message string.
    fn handle(location: &SourceLocation, expression: &str, message: Option<&str>) {
        #[cfg(not(feature = "DEBUG_ASSERT_NO_STDIO"))]
        {
            if expression.is_empty() {
                if let Some(msg) = message {
                    eprintln!(
                        "[debug assert] {}:{}: Unreachable code reached - {}.",
                        location.file_name, location.line_number, msg
                    );
                } else {
                    eprintln!(
                        "[debug assert] {}:{}: Unreachable code reached.",
                        location.file_name, location.line_number
                    );
                }
            } else if let Some(msg) = message {
                eprintln!(
                    "[debug assert] {}:{}: Assertion '{}' failed - {}.",
                    location.file_name, location.line_number, expression, msg
                );
            } else {
                eprintln!(
                    "[debug assert] {}:{}: Assertion '{}' failed.",
                    location.file_name, location.line_number, expression
                );
            }
        }
    }
}

/// \exclude
mod detail {
    use super::*;

    //=== boilerplate ===//
    pub fn forward<T>(t: T) -> T {
        t
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
    pub fn debug_assertion_failed<Handler>(
        location: &SourceLocation,
        expression: &str,
        args: Option<&str>,
    ) where
        Handler: AllowsException,
    {
        if Handler::VALUE {
            DefaultHandler::handle(location, expression, args);
        }
        abort();
    }

    pub fn do_assert<Expr, Handler, const LEVEL: u32>(
        expr: Expr,
        location: &SourceLocation,
        expression: &str,
        _handler: Handler,
    ) -> Option<RegularVoid>
    where
        Expr: Fn() -> bool,
        Handler: AllowsException,
    {
        if LEVEL <= Handler::VALUE as u32 && !expr() {
            debug_assertion_failed::<Handler>(location, expression, None);
        }
        Some(RegularVoid::new())
    }

    pub const fn always_false() -> bool {
        false
    }
}

//=== assertion macros ===//
#[macro_export]
macro_rules! DEBUG_ASSERT {
    ($Expr:expr, $Handler:ident, $Level:expr) => {
        if $Level <= $Handler::VALUE as u32 && !$Expr {
            $Handler::handle(&$crate::DEBUG_ASSERT_CUR_SOURCE_LOCATION!(), stringify!($Expr), None);
            std::process::abort();
        }
    };
}

#[macro_export]
macro_rules! DEBUG_UNREACHABLE {
    ($Handler:ident) => {
        $Handler::handle(&$crate::DEBUG_ASSERT_CUR_SOURCE_LOCATION!(), "", None);
        std::process::abort();
    };
}

fn main() {}