use std::cell::RefCell;
use std::ffi::{CStr, CString};
use std::ptr;
use std::sync::Mutex;
use std::os::raw::c_char;

mod ppk {
    pub mod assert {
        pub mod implementation {
            use std::os::raw::c_char;

            #[derive(Debug, PartialEq, Clone, Copy)]
            pub enum AssertLevel {
                Warning,
                Debug,
                Error,
                Fatal,
            }

            #[derive(Debug, PartialEq, Clone, Copy)]
            pub enum AssertAction {
                None,
                Throw,
                IgnoreLine,
                IgnoreAll,
            }

            pub type AssertHandler = fn(
                file: *const c_char,
                line: i32,
                function: *const c_char,
                expression: *const c_char,
                level: i32,
                message: *const c_char,
            ) -> AssertAction;

            static mut ASSERT_HANDLER: Option<AssertHandler> = None;

            pub fn set_assert_handler(handler: AssertHandler) {
                unsafe {
                    ASSERT_HANDLER = Some(handler);
                }
            }

            pub fn invoke_assert_handler(
                file: *const c_char,
                line: i32,
                function: *const c_char,
                expression: *const c_char,
                level: i32,
                message: *const c_char,
            ) -> AssertAction {
                unsafe {
                    match ASSERT_HANDLER {
                        Some(handler) => handler(file, line, function, expression, level, message),
                        None => AssertAction::None,
                    }
                }
            }

            pub fn ignore_all_asserts(ignore: bool) {
                if ignore {
                    // Perform necessary steps to ignore all asserts
                }
            }
        }
    }
}

use ppk::assert::implementation::{self as implementation, AssertAction, AssertLevel};

thread_local! {
    static FILE: RefCell<*const c_char> = RefCell::new(ptr::null());
    static LINE: RefCell<i32> = RefCell::new(0);
    static FUNCTION: RefCell<*const c_char> = RefCell::new(ptr::null());
    static EXPRESSION: RefCell<*const c_char> = RefCell::new(ptr::null());
    static LEVEL: RefCell<i32> = RefCell::new(0);
    static MESSAGE: RefCell<*mut c_char> = RefCell::new(ptr::null_mut());
}

static ACTION: Mutex<AssertAction> = Mutex::new(AssertAction::None);

fn test_handler(
    file: *const c_char,
    line: i32,
    function: *const c_char,
    expression: *const c_char,
    level: i32,
    message: *const c_char,
) -> AssertAction {
    FILE.with(|f| *f.borrow_mut() = file);
    LINE.with(|l| *l.borrow_mut() = line);
    FUNCTION.with(|func| *func.borrow_mut() = function);
    EXPRESSION.with(|e| *e.borrow_mut() = expression);
    LEVEL.with(|l| *l.borrow_mut() = level);

    MESSAGE.with(|m| {
        if !m.borrow().is_null() {
            unsafe {
                CString::from_raw(*m.borrow_mut());
            }
        }
        if !message.is_null() {
            *m.borrow_mut() = unsafe { CStr::from_ptr(message).to_owned().into_raw() };
        }
    });

    if level == AssertLevel::Error as i32 {
        return AssertAction::Throw;
    }

    *ACTION.lock().unwrap()
}

struct AssertTest;

impl AssertTest {
    fn new() -> Self {
        implementation::set_assert_handler(test_handler);
        *ACTION.lock().unwrap() = AssertAction::None;
        MESSAGE.with(|m| *m.borrow_mut() = ptr::null_mut());
        AssertTest
    }
}

impl Drop for AssertTest {
    fn drop(&mut self) {
        implementation::set_assert_handler(test_handler);
        MESSAGE.with(|m| {
            if !m.borrow().is_null() {
                unsafe {
                    CString::from_raw(*m.borrow_mut());
                }
            }
            *m.borrow_mut() = ptr::null_mut();
        });
    }
}

macro_rules! PPK_ASSERT_WARNING {
    ($cond:expr) => {
        if !$cond {
            implementation::invoke_assert_handler(
                CStr::from_bytes_with_nul(b"ppk_assert_test.cpp\0").unwrap().as_ptr(),
                line!() as i32 - 2,
                CStr::from_bytes_with_nul(b"function\0").unwrap().as_ptr(),
                CStr::from_bytes_with_nul(format!("{}\0", stringify!($cond)).as_bytes()).unwrap().as_ptr(),
                AssertLevel::Warning as i32,
                ptr::null(),
            );
        }
    };
    ($cond:expr, $msg:literal) => {
        if !$cond {
            implementation::invoke_assert_handler(
                CStr::from_bytes_with_nul(b"ppk_assert_test.cpp\0").unwrap().as_ptr(),
                line!() as i32 - 2,
                CStr::from_bytes_with_nul(b"function\0").unwrap().as_ptr(),
                CStr::from_bytes_with_nul(format!("{}\0", stringify!($cond)).as_bytes()).unwrap().as_ptr(),
                AssertLevel::Warning as i32,
                CStr::from_bytes_with_nul(format!("{}\0", $msg).as_bytes()).unwrap().as_ptr(),
            );
        }
    };
}

// Additional similar macros for ASSERT, ASSERT_DEBUG, ASSERT_ERROR, ASSERT_FATAL...

#[test]
fn test_assert_warning() {
    let _test_context = AssertTest::new();

    PPK_ASSERT_WARNING!(true);
    PPK_ASSERT_WARNING!(true, "always true, never fails");

    PPK_ASSERT_WARNING!(false);
    assert_eq!(CStr::from_bytes_with_nul(b"ppk_assert_test.cpp\0").unwrap(), CStr::from_ptr(FILE.with(|f| *f.borrow())));
    assert_eq!(line!() as i32 - 2, LINE.with(|l| *l.borrow()));
    assert_eq!(AssertLevel::Warning as i32, LEVEL.with(|l| *l.borrow()));
    assert_eq!(ptr::null(), MESSAGE.with(|m| *m.borrow()));

    PPK_ASSERT_WARNING!(false, "always false, always fails");
    assert_eq!(CStr::from_bytes_with_nul(b"ppk_assert_test.cpp\0").unwrap(), CStr::from_ptr(FILE.with(|f| *f.borrow())));
    assert_eq!(line!() as i32 - 2, LINE.with(|l| *l.borrow()));
    assert_eq!(AssertLevel::Warning as i32, LEVEL.with(|l| *l.borrow()));
    assert_eq!(CString::new("always false, always fails").unwrap().as_c_str(), unsafe { CStr::from_ptr(MESSAGE.with(|m| *m.borrow())) });
}

// Additional tests for ASSERT, ASSERT_DEBUG, ASSERT_ERROR, ASSERT_FATAL...

fn main() {
    // Testing framework initialization and execution can be placed here.
}