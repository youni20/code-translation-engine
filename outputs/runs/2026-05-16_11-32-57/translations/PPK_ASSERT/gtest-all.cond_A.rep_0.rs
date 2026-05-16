#![allow(dead_code, unused_imports)]
use std::sync::{Arc, Mutex};

#[cfg(windows)]
use std::os::windows::io::RawHandle;

#[cfg(unix)]
use std::os::unix::io::RawFd;

enum InterceptMode {
    InterceptOnlyCurrentThread,
    InterceptAllThreads,
}

struct ScopedFakeTestPartResultReporter {
    intercept_mode: InterceptMode,
    old_reporter: Option<Arc<dyn TestPartResultReporter>>,
    result: Arc<Mutex<TestPartResultArray>>,
}

impl ScopedFakeTestPartResultReporter {
    fn new(result: Arc<Mutex<TestPartResultArray>>) -> Self {
        Self {
            intercept_mode: InterceptMode::InterceptOnlyCurrentThread,
            old_reporter: None,
            result,
        }
    }

    fn new_with_mode(intercept_mode: InterceptMode, result: Arc<Mutex<TestPartResultArray>>) -> Self {
        Self {
            intercept_mode,
            old_reporter: None,
            result,
        }
    }

    fn init(&mut self) {
        let unit_test_impl = UnitTestImpl::get();
        if matches!(self.intercept_mode, InterceptMode::InterceptAllThreads) {
            self.old_reporter = Some(unit_test_impl.lock().unwrap().get_global_test_part_result_reporter());
        } else {
            self.old_reporter = Some(unit_test_impl.lock().unwrap().get_test_part_result_reporter_for_current_thread());
        }
    }

    fn report_test_part_result(&self, result: TestPartResult) {
        self.result.lock().unwrap().append(result);
    }
}

trait TestPartResultReporter {
    fn report_test_part_result(&self, result: TestPartResult);
}

struct DefaultGlobalTestPartResultReporter {
    unit_test: Arc<UnitTestImpl>,
}

impl TestPartResultReporter for DefaultGlobalTestPartResultReporter {
    fn report_test_part_result(&self, result: TestPartResult) {
        // Implementation here
    }
}

struct DefaultPerThreadTestPartResultReporter {
    unit_test: Arc<UnitTestImpl>,
}

impl TestPartResultReporter for DefaultPerThreadTestPartResultReporter {
    fn report_test_part_result(&self, result: TestPartResult) {
        // Implementation here
    }
}

struct UnitTestImpl {
    global_test_part_result_reporter: Option<Arc<dyn TestPartResultReporter>>,
    test_part_result_reporter_for_current_thread: Option<Arc<dyn TestPartResultReporter>>,
}

impl UnitTestImpl {
    fn get() -> Arc<Mutex<UnitTestImpl>> {
        Arc::new(Mutex::new(UnitTestImpl {
            global_test_part_result_reporter: None,
            test_part_result_reporter_for_current_thread: None,
        }))
    }

    fn get_global_test_part_result_reporter(&self) -> Arc<dyn TestPartResultReporter> {
        // Implementation here
        Arc::new(DefaultGlobalTestPartResultReporter {
            unit_test: Arc::new(UnitTestImpl {
                global_test_part_result_reporter: None,
                test_part_result_reporter_for_current_thread: None,
            }),
        })
    }

    fn get_test_part_result_reporter_for_current_thread(&self) -> Arc<dyn TestPartResultReporter> {
        // Implementation here
        Arc::new(DefaultPerThreadTestPartResultReporter {
            unit_test: Arc::new(UnitTestImpl {
                global_test_part_result_reporter: None,
                test_part_result_reporter_for_current_thread: None,
            }),
        })
    }

    fn set_global_test_part_result_reporter(&mut self, reporter: Arc<dyn TestPartResultReporter>) {
        self.global_test_part_result_reporter = Some(reporter);
    }

    fn set_test_part_result_reporter_for_current_thread(&mut self, reporter: Arc<dyn TestPartResultReporter>) {
        self.test_part_result_reporter_for_current_thread = Some(reporter);
    }
}

struct TestPartResultArray {
    results: Vec<TestPartResult>,
}

impl TestPartResultArray {
    fn append(&mut self, result: TestPartResult) {
        self.results.push(result);
    }
}

struct TestPartResult;

fn main() {
    // Instantiate and use the structs here...
}