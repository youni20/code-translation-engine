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

#[cfg(not(DEBUG_ASSERT_NO_STDIO))]
use std::process;

//=== source location ===//

/// Defines a location in the source code.
#[derive(Debug)]
struct SourceLocation {
    file_name: &'static str,
    line_number: u32,
}

/// Expands to the current `SourceLocation`
macro_rules! DEBUG_ASSERT_CUR_SOURCE_LOCATION {
    () => {
        SourceLocation {
            file_name: file!(),
            line_number: line!(),
        }
    };
}

//=== level ===//

/// Tag type to indicate the level of an assertion.
struct Level<const LEVEL: u32>;

/// Helper class that sets a certain level.
struct SetLevel<const LEVEL: u32>;

/// Helper class that controls whether the handler can throw or not.
struct AllowException;

trait IsAllowException {
    const THROWING_EXCEPTION_IS_ALLOWED: bool;
}

impl IsAllowException for AllowException {
    const THROWING_EXCEPTION_IS_ALLOWED: bool = true;
}

//=== handler ===//

/// Does not do anything to handle a failed assertion (except calling `std::process::abort`).
struct NoHandler;

impl NoHandler {
    /// Does nothing.
    fn handle(location: &SourceLocation, expression: &str, args: impl std::fmt::Debug) {
        let _ = (location, expression, args);
    }
}

/// The default handler that writes a message to `stderr`.
struct DefaultHandler;

impl DefaultHandler {
    fn handle(location: &SourceLocation, expression: &str, message: Option<&str>) {
        #[cfg(not(DEBUG_ASSERT_NO_STDIO))]
        {
            if expression.is_empty() {
                if let Some(msg) = message {
                    eprintln!(
                        "[debug assert] {}: {}: Unreachable code reached - {}.",
                        location.file_name, location.line_number, msg
                    );
                } else {
                    eprintln!(
                        "[debug assert] {}: {}: Unreachable code reached.",
                        location.file_name, location.line_number
                    );
                }
            } else if let Some(msg) = message {
                eprintln!(
                    "[debug assert] {}: {}: Assertion '{}' failed - {}.",
                    location.file_name, location.line_number, expression, msg
                );
            } else {
                eprintln!(
                    "[debug assert] {}: {}: Assertion '{}' failed.",
                    location.file_name, location.line_number, expression
                );
            }
        }
    }
}

//=== regular void fake ===//
struct RegularVoid;

impl RegularVoid {
    const fn new() -> Self {
        RegularVoid
    }
}

//=== assert implementation ===//

fn debug_assertion_failed<H: IsAllowException>(
    location: &SourceLocation,
    expression: &str,
    args: impl std::fmt::Debug,
) -> RegularVoid {
    DefaultHandler::handle(location, expression, Some(&format!("{:?}", args)));
    process::abort();
}

fn do_assert<Expr, H: IsAllowException, const LEVEL: u32>(
    expr: Expr,
    location: &SourceLocation,
    expression: &str,
    _handler: H,
    _level: Level<LEVEL>,
) -> RegularVoid
where
    Expr: Fn() -> bool,
{
    if LEVEL <= H::THROWING_EXCEPTION_IS_ALLOWED as u32 {
        if expr() {
            RegularVoid::new()
        } else {
            debug_assertion_failed::<H>(location, expression, ())
        }
    } else {
        RegularVoid::new()
    }
}

//=== assertion macros ===//

#[macro_export]
macro_rules! debug_assert {
    ($expr:expr, $handler:expr, $($arg:expr),*) => {
        if !$expr {
            $handler.handle(
                DEBUG_ASSERT_CUR_SOURCE_LOCATION!(),
                stringify!($expr),
                format_args!($($arg),*)
            )
        }
    };
}

#[macro_export]
macro_rules! debug_unreachable {
    ($handler:expr, $($arg:expr),*) => {{
        $handler.handle(
            DEBUG_ASSERT_CUR_SOURCE_LOCATION!(),
            "",
            format_args!($($arg),*)
        );
        process::abort();
    }};
}

fn main() {
    // Example usage in main for testing the handlers.
    let location = DEBUG_ASSERT_CUR_SOURCE_LOCATION!();
    DefaultHandler::handle(&location, "x > 0", Some("test failed"));
}