use std::cell::RefCell;
use std::ffi::CString;

// Simulate the original C++ names
mod ppk {
    use super::AssertAction;
    use std::sync::Mutex;
    use std::sync::OnceLock;

    pub mod assert {
        pub mod implementation {
            use super::super::AssertAction;
            use std::sync::Mutex;
            use std::sync::OnceLock;
          
            pub static ASSERT_HANDLER: OnceLock<Mutex<Box<fn(&str, i32, &str, &str, i32, Option<&str>) -> AssertAction>>> = OnceLock::new();

            pub fn set_assert_handler(
                handler: fn(&str, i32, &str, &str, i32, Option<&str>) -> AssertAction,
            ) {
                let mutex = ASSERT_HANDLER.get_or_init(|| Mutex::new(Box::new(handler)));
                let mut default_handler = mutex.lock().unwrap();
                *default_handler = Box::new(handler);
            }

            pub fn ignore_all_asserts(_flag: bool) {
                // Implementation goes here for ignoring all asserts
            }
        }
    }
}

// Enums to emulate different assertion levels and actions
#[derive(Debug, PartialEq, Clone)]
pub enum AssertLevel {
    Warning,
    Debug,
    Error,
    Fatal,
}

#[derive(Debug, PartialEq, Clone)]
pub enum AssertAction {
    None,
    Throw,
    IgnoreLine,
    IgnoreAll,
}

// Exception structure compatible with the C++
pub struct AssertionException;

// Test state is stored in thread-local storage
thread_local! {
    static TEST_STATE: RefCell<TestState> = RefCell::new(TestState::new());
}

// State to hold current assertion details
struct TestState {
    file: Option<String>,
    line: i32,
    function: Option<String>,
    expression: Option<String>,
    level: i32,
    message: Option<CString>,
    action: AssertAction,
}

impl TestState {
    fn new() -> Self {
        Self {
            file: None,
            line: 0,
            function: None,
            expression: None,
            level: 0,
            message: None,
            action: AssertAction::None,
        }
    }
}

fn test_handler(
    file: &str,
    line: i32,
    function: &str,
    expression: &str,
    level: i32,
    message: Option<&str>,
) -> AssertAction {
    TEST_STATE.with(|state| {
        let mut state = state.borrow_mut();

        state.file = Some(file.to_string());
        state.line = line;
        state.function = Some(function.to_string());
        state.expression = Some(expression.to_string());
        state.level = level;

        if let Some(msg) = message {
            state.message = Some(CString::new(msg).unwrap());
        } else {
            state.message = None;
        }

        if level == AssertLevel::Error as i32 {
            return AssertAction::Throw; // Simulate throwing exception
        }

        state.action.clone()
    })
}

/// Emulating Macro Definition for `PPK_ASSERT`
fn ppk_assert(condition: bool, msg: Option<&str>, level: AssertLevel) {
    if !condition {
        if let Some(handler) = ppk::assert::implementation::ASSERT_HANDLER.get() {
            let handler = handler.lock().unwrap();
            handler(
                "ppk_assert_test.rs",
                line!() as i32,
                std::any::type_name::<fn()>(),
                "expression",
                level as i32,
                msg,
            );
        }
    }
}

macro_rules! PPK_ASSERT_WARNING {
    ($cond:expr) => {
        ppk_assert($cond, None, AssertLevel::Warning)
    };
    ($cond:expr, $msg:expr) => {
        ppk_assert($cond, Some($msg), AssertLevel::Warning)
    };
}

macro_rules! PPK_ASSERT {
    ($cond:expr) => {
        ppk_assert($cond, None, AssertLevel::Debug)
    };
    ($cond:expr, $msg:expr) => {
        ppk_assert($cond, Some($msg), AssertLevel::Debug)
    };
}

// Setup testing framework
#[cfg(test)]
mod tests {
    use super::*;

    struct AssertTest;

    impl AssertTest {
        fn new() -> Self {
            ppk::assert::implementation::set_assert_handler(test_handler);
            TEST_STATE.with(|state| {
                let mut state = state.borrow_mut();
                state.action = AssertAction::None;
                state.message = None;
            });

            AssertTest
        }
    }

    impl Drop for AssertTest {
        fn drop(&mut self) {
            ppk::assert::implementation::set_assert_handler(test_handler);
            TEST_STATE.with(|state| {
                let mut state = state.borrow_mut();
                if let Some(_) = &state.message {
                    state.message = None;
                }
            });
        }
    }

    #[test]
    fn assert_warning() {
        let _ = AssertTest::new();

        PPK_ASSERT_WARNING!(true);
        PPK_ASSERT_WARNING!(true, "always true, never fails");

        PPK_ASSERT_WARNING!(false);
        TEST_STATE.with(|state| {
            let state = state.borrow();
            assert_eq!(state.file.as_deref(), Some("ppk_assert_test.rs"));
            assert_eq!(state.level, AssertLevel::Warning as i32);
            assert!(state.message.is_none());
        });

        PPK_ASSERT_WARNING!(false, "always false, always fails");
        TEST_STATE.with(|state| {
            let state = state.borrow();
            assert_eq!(state.message.as_deref().map(|cstr| cstr.to_str().unwrap()), Some("always false, always fails"));
        });
    }
}

fn main() {
    // The Rust test runner will handle running the tests defined above
}