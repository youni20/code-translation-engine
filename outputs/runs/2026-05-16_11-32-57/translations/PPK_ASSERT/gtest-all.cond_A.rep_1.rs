use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy, PartialEq)]
enum InterceptMode {
    InterceptOnlyCurrentThread,
    InterceptAllThreads,
}

struct ScopedFakeTestPartResultReporter {
    intercept_mode: InterceptMode,
    old_reporter: Option<Arc<Mutex<TestPartResultReporterInterface>>>,
    result: Arc<Mutex<TestPartResultArray>>,
}

impl ScopedFakeTestPartResultReporter {
    fn new(intercept_mode: InterceptMode, result: Arc<Mutex<TestPartResultArray>>) -> Self {
        let old_reporter = {
            let impl_ = get_unit_test_impl();
            if intercept_mode == InterceptMode::InterceptAllThreads {
                let old = impl_.lock().unwrap().get_global_test_part_result_reporter();
                impl_.lock().unwrap().set_global_test_part_result_reporter(
                    Arc::new(Mutex::new(TestPartResultReporterInterface)),
                );
                old
            } else {
                let old = impl_.lock().unwrap().get_test_part_result_reporter_for_current_thread();
                impl_.lock().unwrap().set_test_part_result_reporter_for_current_thread(
                    Arc::new(Mutex::new(TestPartResultReporterInterface)),
                );
                old
            }
        };
        ScopedFakeTestPartResultReporter {
            intercept_mode,
            old_reporter: Some(old_reporter),
            result,
        }
    }
}

impl Drop for ScopedFakeTestPartResultReporter {
    fn drop(&mut self) {
        let impl_ = get_unit_test_impl();
        if self.intercept_mode == InterceptMode::InterceptAllThreads {
            if let Some(old) = &self.old_reporter {
                impl_.lock().unwrap().set_global_test_part_result_reporter(old.clone());
            }
        } else {
            if let Some(old) = &self.old_reporter {
                impl_.lock().unwrap().set_test_part_result_reporter_for_current_thread(old.clone());
            }
        }
    }
}

struct TestPartResultReporterInterface;

struct TestPartResultReporter {
    // Your implementation here...
}

impl TestPartResultReporter {
    fn new() -> Self {
        TestPartResultReporter {
            // Your fields here...
        }
    }
}

struct TestPartResultArray {
    // Your implementation here...
}

fn get_unit_test_impl() -> Arc<Mutex<UnitTestImpl>> {
    // Your singleton implementation here...
    Arc::new(Mutex::new(UnitTestImpl::new()))
}

struct UnitTestImpl {
    // Your fields here...
    global_reporter: Option<Arc<Mutex<TestPartResultReporterInterface>>>,
    thread_reporter: Arc<Mutex<TestPartResultReporterInterface>>,
}

impl UnitTestImpl {
    fn new() -> Self {
        UnitTestImpl {
            global_reporter: None,
            thread_reporter: Arc::new(Mutex::new(TestPartResultReporterInterface)),
        }
    }
    
    fn get_global_test_part_result_reporter(&self) -> Arc<Mutex<TestPartResultReporterInterface>> {
        self.global_reporter.clone().unwrap_or_else(|| Arc::new(Mutex::new(TestPartResultReporterInterface)))
    }
    
    fn set_global_test_part_result_reporter(&mut self, reporter: Arc<Mutex<TestPartResultReporterInterface>>) {
        self.global_reporter = Some(reporter);
    }
    
    fn get_test_part_result_reporter_for_current_thread(&self) -> Arc<Mutex<TestPartResultReporterInterface>> {
        self.thread_reporter.clone()
    }
    
    fn set_test_part_result_reporter_for_current_thread(&mut self, reporter: Arc<Mutex<TestPartResultReporterInterface>>) {
        self.thread_reporter = reporter;
    }
}

struct SingleFailureChecker {
    // Fields and methods...
}

impl SingleFailureChecker {
    fn new(_results: Arc<Mutex<TestPartResultArray>>, _result_type: i32, _substr: &str) -> Self {
        SingleFailureChecker {
            // Initialization...
        }
    }
}

fn main() {
    // Your test code here...
}