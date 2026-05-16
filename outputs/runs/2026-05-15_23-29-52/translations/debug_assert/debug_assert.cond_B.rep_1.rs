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

use std::process::abort;

#[derive(Debug, Clone, Copy)]
pub struct SourceLocation {
    pub file_name: &'static str,
    pub line_number: u32,
}

#[macro_export]
macro_rules! DEBUG_ASSERT_CUR_SOURCE_LOCATION {
    () => {
        SourceLocation {
            file_name: file!(),
            line_number: line!(),
        }
    };
}

pub trait Level {
    fn level() -> u32;
}

pub struct SetLevel<const N: u32>;

impl<const N: u32> Level for SetLevel<N> {
    fn level() -> u32 {
        N
    }
}

pub struct AllowException;

impl AllowException {
    pub const THROWING_EXCEPTION_IS_ALLOWED: bool = true;
}

pub struct NoHandler;

impl NoHandler {
    pub fn handle(_: &SourceLocation, _: &str) {}
}

pub struct DefaultHandler;

impl DefaultHandler {
    pub fn handle(loc: &SourceLocation, expression: &str, message: Option<&str>) {
        let formatted_message = if expression.is_empty() {
            match message {
                Some(msg) => format!(
                    "[debug assert] {}:{}: Unreachable code reached - {}.\n",
                    loc.file_name, loc.line_number, msg
                ),
                None => format!(
                    "[debug assert] {}:{}: Unreachable code reached.\n",
                    loc.file_name, loc.line_number
                ),
            }
        } else {
            match message {
                Some(msg) => format!(
                    "[debug assert] {}:{}: Assertion '{}' failed - {}.\n",
                    loc.file_name, loc.line_number, expression, msg
                ),
                None => format!(
                    "[debug assert] {}:{}: Assertion '{}' failed.\n",
                    loc.file_name, loc.line_number, expression
                ),
            }
        };

        eprintln!("{}", formatted_message);
    }
}

mod detail {
    use super::{abort, SourceLocation};

    pub struct RegularVoid;

    pub fn forward<T>(t: T) -> T {
        t
    }

    pub fn debug_assertion_failed<H, A>(
        loc: &SourceLocation,
        expression: &str,
        handler: H,
        args: A,
    ) -> RegularVoid
    where
        H: Fn(&SourceLocation, &str, A),
    {
        handler(loc, expression, args);
        abort();
    }

    pub const fn always_false() -> bool {
        false
    }

    pub fn do_assert<E, H, F>(
        expr: E,
        loc: SourceLocation,
        expression: &'static str,
        handler: H,
        args: F,
    ) -> RegularVoid
    where
        E: Fn() -> bool,
        H: Fn(&SourceLocation, &str, F),
        F: Clone,
    {
        if expr() {
            RegularVoid
        } else {
            debug_assertion_failed(&loc, expression, handler, args)
        }
    }
}

#[macro_export]
macro_rules! DEBUG_ASSERT {
    ($expr:expr, $handler:expr, $($args:expr),*) => {
        detail::do_assert(
            || $expr,
            DEBUG_ASSERT_CUR_SOURCE_LOCATION!(),
            stringify!($expr),
            $handler,
            ($($args,)*),
        )
    };
}

#[macro_export]
macro_rules! DEBUG_UNREACHABLE {
    ($handler:expr, $($args:expr),*) => {
        detail::do_assert(
            detail::always_false,
            DEBUG_ASSERT_CUR_SOURCE_LOCATION!(),
            "",
            $handler,
            ($($args,)*),
        )
    };
}

fn main() {
    let loc = DEBUG_ASSERT_CUR_SOURCE_LOCATION!();
    assert_eq!(loc.file_name, file!());
}