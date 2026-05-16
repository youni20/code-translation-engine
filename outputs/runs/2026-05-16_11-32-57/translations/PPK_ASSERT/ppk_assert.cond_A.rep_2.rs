use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

#[derive(Debug, Clone, Copy)]
pub enum AssertLevel {
    Warning,
    Debug,
    Error,
    Fatal,
}

#[derive(PartialEq)]
pub enum AssertAction {
    None,
    Abort,
    Break,
    Ignore,
    IgnoreAll,
    Throw,
}

#[derive(Debug)]
pub struct AssertionException {
    file: &'static str,
    line: u32,
    function: &'static str,
    expression: &'static str,
    message: String,
}

impl AssertionException {
    pub fn new(
        file: &'static str,
        line: u32,
        function: &'static str,
        expression: &'static str,
        message: &str,
    ) -> Self {
        let formatted_message = format!(
            "Assertion failed at {}:{} in {}: {}: {}",
            file, line, function, expression, message
        );
        AssertionException {
            file,
            line,
            function,
            expression,
            message: formatted_message,
        }
    }
}

impl fmt::Display for AssertionException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for AssertionException {}

pub type AssertHandler = fn(
    file: &'static str,
    line: u32,
    function: &'static str,
    expression: &'static str,
    level: AssertLevel,
    message: &str,
) -> AssertAction;

static ASSERT_HANDLER: Mutex<AssertHandler> = Mutex::new(default_assert_handler);
static IGNORE_ALL_ASSERTS: AtomicBool = AtomicBool::new(false);

pub fn set_assert_handler(handler: AssertHandler) {
    let mut guard = ASSERT_HANDLER.lock().unwrap();
    *guard = handler;
}

pub fn ignore_all_asserts(value: bool) {
    IGNORE_ALL_ASSERTS.store(value, Ordering::SeqCst);
}

pub fn ignore_all_asserts_state() -> bool {
    IGNORE_ALL_ASSERTS.load(Ordering::SeqCst)
}

fn default_assert_handler(
    file: &'static str,
    line: u32,
    function: &'static str,
    expression: &'static str,
    level: AssertLevel,
    message: &str,
) -> AssertAction {
    eprintln!(
        "[{:?}] Assertion failed at {}:{} in {}: {} - {}",
        level, file, line, function, expression, message
    );
    AssertAction::Abort
}

macro_rules! PPK_ASSERT {
    ($level:expr, $expression:expr $(, $message:expr)?) => {
        if !$expression && !ignore_all_asserts_state() {
            let message = format_args!($($message,)*).to_string();
            let handler = ASSERT_HANDLER.lock().unwrap();
            if handler(file!(), line!(), "", stringify!($expression), $level, &message) == AssertAction::Break {
                debug_break();
            }
        }
    }
}

macro_rules! PPK_STATIC_ASSERT {
    ($expression:expr) => {
        const _: fn() = || {
            if !$expression {
                panic!("Static assertion failed");
            }
        };
    };
    ($expression:expr, $message:expr) => {
        const _: fn() = || {
            if !$expression {
                panic!($message);
            }
        };
    };
}

fn debug_break() {
    #[cfg(target_os = "windows")]
    {
        unsafe { core::intrinsics::breakpoint() }
    }
    #[cfg(not(target_os = "windows"))]
    {
        unsafe { std::arch::asm!("int3") }
    }
}

macro_rules! PPK_ASSERT_JOIN {
    ($lhs:ident, $rhs:ident) => {
        concat_idents!($lhs, $rhs)
    };
}

#[macro_export]
macro_rules! concat_idents {
    ($x:ident, $y:ident) => {
        concat!(stringify!($x), stringify!($y))
    };
}

pub struct AssertUsedWrapper<T> {
    value: T,
    used: AtomicBool,
    level: AssertLevel,
}

impl<T> AssertUsedWrapper<T> {
    pub fn new(value: T, level: AssertLevel) -> Self {
        AssertUsedWrapper {
            value,
            used: AtomicBool::new(false),
            level,
        }
    }

    pub fn get(&self) -> &T {
        self.used.store(true, Ordering::SeqCst);
        &self.value
    }
}

impl<T> Drop for AssertUsedWrapper<T> {
    fn drop(&mut self) {
        if !self.used.load(Ordering::SeqCst) {
            PPK_ASSERT!(self.level, false, "Unused value detected");
        }
    }
}

fn main() {
    // Add your main functionality or tests here
}