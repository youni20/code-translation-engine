use std::io::Write;

//=== source location ===//
#[derive(Copy, Clone)]
pub struct SourceLocation {
    pub file_name: &'static str,
    pub line_number: u32,
}

macro_rules! DEBUG_ASSERT_CUR_SOURCE_LOCATION {
    () => {
        SourceLocation {
            file_name: file!(),
            line_number: line!(),
        }
    };
}

//=== level ===//
pub struct Level<const Level: u32>;

pub struct SetLevel<const Level: u32>;

impl<const L: u32> SetLevel<L> {
    pub const LEVEL: u32 = L;
}

pub struct AllowException;

impl AllowException {
    pub const THROWING_EXCEPTION_IS_ALLOWED: bool = true;
}

//=== handler ===//
pub struct NoHandler;

impl NoHandler {
    pub fn handle(_loc: &SourceLocation, _expression: &str, _args: &str) {
    }
}

pub struct DefaultHandler;

impl DefaultHandler {
    pub fn handle(loc: &SourceLocation, expression: &str, message: Option<&str>) {
        #[cfg(not(feature = "DEBUG_ASSERT_NO_STDIO"))]
        {
            let mut stderr = std::io::stderr();
            let _ = if expression.is_empty() {
                if let Some(msg) = message {
                    writeln!(
                        stderr,
                        "[debug assert] {}:{}: Unreachable code reached - {}.",
                        loc.file_name, loc.line_number, msg
                    )
                } else {
                    writeln!(
                        stderr,
                        "[debug assert] {}:{}: Unreachable code reached.",
                        loc.file_name, loc.line_number
                    )
                }
            } else if let Some(msg) = message {
                writeln!(
                    stderr,
                    "[debug assert] {}:{}: Assertion '{}' failed - {}.",
                    loc.file_name, loc.line_number, expression, msg
                )
            } else {
                writeln!(
                    stderr,
                    "[debug assert] {}:{}: Assertion '{}' failed.",
                    loc.file_name, loc.line_number, expression
                )
            };
        }
    }
}

/// \exclude
pub mod detail {
    use std::process::abort;

    pub trait RemoveReference {}

    impl<T> RemoveReference for T {}

    pub fn forward<T: RemoveReference>(t: T) -> T {
        t
    }    

    pub trait EnableIf<const CONDITION: bool> {}

    impl<T> EnableIf<true> for T {}

    impl<T> EnableIf<false> for T {}

    pub struct AllowsException<const VALUE: bool>;

    pub struct RegularVoid;

    impl RegularVoid {
        pub fn new() -> Self {
            RegularVoid
        }
    }

    pub fn debug_assertion_failed<Handler, Args: std::fmt::Debug>(
        loc: &super::SourceLocation,
        expression: &str,
        args: Args,
    ) -> RegularVoid
    where
        Handler: Default + Fn(&super::SourceLocation, &str, &str),
    {
        Handler::default()(loc, expression, &format!("{:?}", args));
        abort();
    }

    pub fn do_assert<Expr, Handler, Args>(
        expr: Expr,
        loc: &super::SourceLocation,
        expression: &str,
        _handler: Handler,
        level: u32,
        args: Args,
    ) -> Result<RegularVoid, RegularVoid>
    where
        Expr: Fn() -> bool,
        Handler: Default + Fn(&super::SourceLocation, &str, &str),
        Args: std::fmt::Debug,
    {
        if level <= super::SetLevel::<0>::LEVEL {
            if expr() {
                Ok(RegularVoid::new())
            } else {
                Err(debug_assertion_failed::<Handler, Args>(loc, expression, args))
            }
        } else {
            Ok(RegularVoid::new())
        }
    }

    pub fn always_false() -> bool {
        false
    }
}

//=== assertion macros ===//
macro_rules! DEBUG_ASSERT {
    ($Expr:expr, $Handler:ty, $Level:ty, $($args:expr),*) => {
        {
            let expr_fn = || -> bool { $Expr };
            let loc = DEBUG_ASSERT_CUR_SOURCE_LOCATION!();
            let handler: $Handler = Default::default();
            let level: u32 = <$Level>::LEVEL;
            if expr_fn() {
                let _ = debug_assert::detail::do_assert::<_, $Handler, _>(
                    expr_fn, &loc, stringify!($Expr), handler, level, format!($($args),*)
                );
            }
        }
    };
}

macro_rules! DEBUG_UNREACHABLE {
    ($Handler:ty, $Level:ty, $($args:expr),*) => {
        {
            let loc = DEBUG_ASSERT_CUR_SOURCE_LOCATION!();
            let handler: $Handler = Default::default();
            let level: u32 = <$Level>::LEVEL;
            let expr = || -> bool { debug_assert::detail::always_false() };
            let _ = debug_assert::detail::do_assert::<_, $Handler, _>(
                expr, &loc, "", handler, level, format!($($args),*)
            );
        }
    };
}

fn main() {}