use std::fmt::{self, Display, Formatter};

#[derive(Clone, Copy)]
pub struct SourceLocation {
    file_name: &'static str,
    line_number: u32,
}

impl SourceLocation {
    pub const fn new(file_name: &'static str, line_number: u32) -> Self {
        SourceLocation {
            file_name,
            line_number,
        }
    }
}

impl Display for SourceLocation {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.file_name, self.line_number)
    }
}

#[macro_export]
macro_rules! debug_assert_cur_source_location {
    () => {
        SourceLocation::new(file!(), line!())
    };
}

pub struct Level<const LEVEL: u32>;

pub struct SetLevel<const LEVEL: u32>;

impl<const LEVEL: u32> SetLevel<LEVEL> {
    pub const LEVEL: u32 = LEVEL;
}

pub struct AllowException;

impl AllowException {
    pub const THROWING_EXCEPTION_IS_ALLOWED: bool = true;
}

pub struct NoHandler;

impl NoHandler {
    pub fn handle<'a, Args: fmt::Display>(
        _loc: SourceLocation,
        _expression: &'a str,
        _args: Args,
    ) -> Result<(), &'static str> {
        Ok(())
    }
}

pub struct DefaultHandler;

impl DefaultHandler {
    pub fn handle(
        loc: SourceLocation,
        expression: &str,
        message: Option<&str>,
    ) -> Result<(), &'static str> {
        #[cfg(not(feature = "DEBUG_ASSERT_NO_STDIO"))]
        {
            match (expression.is_empty(), message) {
                (true, Some(msg)) => {
                    eprintln!("[debug assert] {}: Unreachable code reached - {}.", loc, msg);
                }
                (true, None) => {
                    eprintln!("[debug assert] {}: Unreachable code reached.", loc);
                }
                (false, Some(msg)) => {
                    eprintln!(
                        "[debug assert] {}: Assertion '{}' failed - {}.",
                        loc, expression, msg
                    );
                }
                (false, None) => {
                    eprintln!("[debug assert] {}: Assertion '{}' failed.", loc, expression);
                }
            }
        }
        Ok(())
    }
}

pub mod detail {
    use super::*;

    pub struct RegularVoid;

    pub struct EnableIf<const VALUE: bool>;

    pub struct AllowsException<H>(std::marker::PhantomData<H>);

    impl<H> AllowsException<H> {
        pub const DEFAULT_VALUE: bool = false;
    }

    impl AllowsException<AllowException> {
        pub const ALLOWED_VALUE: bool = AllowException::THROWING_EXCEPTION_IS_ALLOWED;
    }

    pub fn debug_assertion_failed<Args: fmt::Display>(
        loc: SourceLocation,
        expression: &str,
        args: Args,
    ) -> ! {
        NoHandler::handle(loc, expression, args).unwrap();
        std::process::abort();
    }

    pub fn do_assert<Expr, H, Args>(
        expr: Expr,
        loc: SourceLocation,
        expression: &'static str,
        _handler: H,
        _level: Level<1>,
        args: Args,
    ) -> Result<RegularVoid, &'static str>
    where
        Expr: Fn() -> bool,
        H: fmt::Debug + Default,
        Args: fmt::Display,
    {
        if expr() {
            Ok(RegularVoid)
        } else {
            debug_assertion_failed::<Args>(loc, expression, args);
        }
    }

    pub const fn always_false() -> bool {
        false
    }
}

#[macro_export]
macro_rules! debug_assert {
    ($expr:expr, $handler:ty $(, $level:expr)? $(, $($arg:tt)*)?) => {{
        detail::do_assert(
            || $expr,
            debug_assert_cur_source_location!(),
            stringify!($expr),
            <$handler>::default(),
            $(<$level>::default())?,
            ($($($arg)*)?)
        ).ok();
    }};
}

#[macro_export]
macro_rules! debug_unreachable {
    ($handler:ty $(, $level:expr)? $(, $($arg:tt)*)?) => {{
        detail::do_assert(
            || detail::always_false(),
            debug_assert_cur_source_location!(),
            "",
            <$handler>::default(),
            $(<$level>::default())?,
            ($($($arg)*)?)
        ).ok();
    }};
}

fn main() {}