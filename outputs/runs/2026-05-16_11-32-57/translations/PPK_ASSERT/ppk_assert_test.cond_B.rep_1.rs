#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::ffi::{CString, CStr};
    use std::fmt;
    use std::panic;

    thread_local! {
        static STATE: RefCell<TestState> = RefCell::new(TestState {
            file: None,
            line: 0,
            function: None,
            expression: None,
            level: AssertLevel::Warning,
            message: None,
        });
    }

    struct TestState {
        file: Option<&'static str>,
        line: u32,
        function: Option<&'static str>,
        expression: Option<&'static str>,
        level: AssertLevel,
        message: Option<CString>,
    }

    #[derive(Debug, PartialEq, Eq, Copy, Clone)]
    #[repr(i32)]
    enum AssertLevel {
        Warning = 1,
        Debug = 2,
        Error = 3,
        Fatal = 4,
        Custom(i32),
    }

    impl fmt::Display for AssertLevel {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match *self {
                AssertLevel::Custom(level) => write!(f, "{}", level),
                _ => write!(f, "{:?}", self),
            }
        }
    }

    #[derive(Debug, PartialEq, Eq)]
    enum AssertAction {
        None,
        IgnoreLine,
        IgnoreAll,
        Throw,
    }

    impl<'a> From<AssertLevel> for &'a str {
        fn from(level: AssertLevel) -> &'a str {
            match level {
                AssertLevel::Warning => "Warning",
                AssertLevel::Debug => "Debug",
                AssertLevel::Error => "Error",
                AssertLevel::Fatal => "Fatal",
                AssertLevel::Custom(_) => "Custom",
            }
        }
    }

    fn test_handler(
        file: &'static str,
        line: u32,
        function: &'static str,
        expression: &'static str,
        level: AssertLevel,
        message: Option<CString>,
    ) -> AssertAction {
        STATE.with(|state| {
            let mut state = state.borrow_mut();
            state.file = Some(file);
            state.line = line;
            state.function = Some(function);
            state.expression = Some(expression);
            state.level = level;
            if let Some(msg) = message {
                state.message = Some(msg);
            }
        });

        if level == AssertLevel::Error {
            return AssertAction::Throw;
        }
        AssertAction::None
    }

    const PPK_ASSERT_LINE: u32 = line!();

    fn assert_warning(condition: bool, message: Option<&str>) {
        assert_condition(AssertLevel::Warning, condition, message);
    }

    fn assert_debug(condition: bool, message: Option<&str>) {
        assert_condition(AssertLevel::Debug, condition, message);
    }

    fn assert_error(condition: bool, message: Option<&str>) {
        assert_condition(AssertLevel::Error, condition, message);
    }

    fn assert_fatal(condition: bool, message: Option<&str>) {
        assert_condition(AssertLevel::Fatal, condition, message);
    }

    fn assert_custom(level: i32, condition: bool, message: Option<CString>) {
        assert_condition(AssertLevel::Custom(level), condition, message.as_deref().map(|cstr| cstr.to_str().unwrap()));
    }

    fn assert_condition(level: AssertLevel, condition: bool, message: Option<&str>) {
        if !condition {
            let file = "ppk_assert_test.rs";
            let line = line!();
            let function = "assert_condition";
            let expression = "expression";

            let c_message = message.map(CString::new).transpose().unwrap();

            let action = test_handler(file, line, function, expression, level, c_message);

            match action {
                AssertAction::Throw => panic!(),
                _ => {}
            }
        }
    }

    #[test]
    fn test_assert_warning() {
        assert_warning(true, None);
        assert_warning(true, Some("always true, never fails"));

        assert_warning(false, None);
        STATE.with(|state| {
            let state = state.borrow();
            assert_eq!(Some("ppk_assert_test.rs"), state.file);
            assert_eq!(PPK_ASSERT_LINE - 6, state.line);
            assert_eq!(AssertLevel::Warning, state.level);
            assert!(state.message.is_none());
        });

        assert_warning(false, Some("always false, always fails"));
        STATE.with(|state| {
            let state = state.borrow();
            assert_eq!(Some("ppk_assert_test.rs"), state.file);
            assert_eq!(PPK_ASSERT_LINE - 6, state.line);
            assert_eq!(AssertLevel::Warning, state.level);
            assert_eq!(Some("always false, always fails"), state.message.as_ref().map(|cstr| cstr.to_str().unwrap()));
        });

        let s = "foo";
        let i = 123;
        let f = 123.456_f32;
        assert_warning(false, Some(&format!("always false, always fails -- s: {}, i: {}, f: {:3.3}", s, i, f)));
        STATE.with(|state| {
            let state = state.borrow();
            assert_eq!(Some("ppk_assert_test.rs"), state.file);
            assert_eq!(AssertLevel::Warning, state.level);
            assert_eq!(Some("always false, always fails -- s: foo, i: 123, f: 123.456"), state.message.as_ref().map(|cstr| cstr.to_str().unwrap()));
        });
    }

    struct Struct {
        count: i32,
    }

    impl Default for Struct {
        fn default() -> Self {
            Struct { count: 0 }
        }
    }

    fn test_struct_used() -> Struct {
        Struct::default()
    }

    #[test]
    fn test_assert_used() {
        let _: Struct = test_struct_used();
        STATE.with(|state| {
            let state = state.borrow();
            assert_eq!(AssertLevel::Debug, state.level);
            assert!(state.message.is_some());
        });
    }
}
fn main() {}