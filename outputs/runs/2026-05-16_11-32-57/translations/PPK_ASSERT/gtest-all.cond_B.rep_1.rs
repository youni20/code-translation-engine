mod google_test_fused {
    pub mod gtest {
        pub mod gtest_spy {
            use std::sync::{Arc, Mutex};

            type TestPartResultReporter = Arc<Mutex<dyn TestPartResultReporterInterface>>;
            type TestPartResultArray = Arc<Mutex<Vec<TestPartResult>>>;

            #[derive(Clone, PartialEq)]
            pub enum Type {
                Success,
                FatalFailure,
                NonFatalFailure,
            }

            #[derive(Clone)]
            pub struct TestPartResult {
                pub type_: Type,
                pub message: String,
            }

            pub struct ScopedFakeTestPartResultReporter {
                intercept_mode: InterceptMode,
                result: TestPartResultArray,
                old_reporter: Option<TestPartResultReporter>,
            }

            impl ScopedFakeTestPartResultReporter {
                pub fn new(result: TestPartResultArray) -> Self {
                    let mut reporter = ScopedFakeTestPartResultReporter {
                        intercept_mode: InterceptMode::OnlyCurrentThread,
                        result: result.clone(),
                        old_reporter: None,
                    };
                    {
                        let _impl_ = get_unit_test_impl().lock().unwrap();
                        reporter.register_custom_reporter();
                    }
                    reporter
                }

                pub fn new_with_mode(intercept_mode: InterceptMode, result: TestPartResultArray) -> Self {
                    let mut reporter = ScopedFakeTestPartResultReporter {
                        intercept_mode,
                        result: result.clone(),
                        old_reporter: None,
                    };
                    {
                        let _impl_ = get_unit_test_impl().lock().unwrap();
                        reporter.register_custom_reporter();
                    }
                    reporter
                }

                fn register_custom_reporter(&mut self) {
                    let mut impl_ = get_unit_test_impl().lock().unwrap();
                    match self.intercept_mode {
                        InterceptMode::OnlyCurrentThread => {
                            self.old_reporter = Some(impl_.set_test_part_result_reporter_for_current_thread(Arc::new(Mutex::new(self.clone()))));
                        }
                        InterceptMode::AllThreads => {
                            self.old_reporter = Some(impl_.set_global_test_part_result_reporter(Arc::new(Mutex::new(self.clone()))));
                        }
                    }
                }
            }

            impl TestPartResultReporterInterface for ScopedFakeTestPartResultReporter {
                fn report_test_part_result(&self, result: TestPartResult) {
                    let mut res = self.result.lock().unwrap();
                    res.push(result);
                }
            }

            impl Drop for ScopedFakeTestPartResultReporter {
                fn drop(&mut self) {
                    let mut impl_ = get_unit_test_impl().lock().unwrap();
                    match self.intercept_mode {
                        InterceptMode::OnlyCurrentThread => {
                            if let Some(old_reporter) = self.old_reporter.take() {
                                impl_.set_test_part_result_reporter_for_current_thread(old_reporter);
                            }
                        }
                        InterceptMode::AllThreads => {
                            if let Some(old_reporter) = self.old_reporter.take() {
                                impl_.set_global_test_part_result_reporter(old_reporter);
                            }
                        }
                    }
                }
            }

            #[derive(Copy, Clone)]
            pub enum InterceptMode {
                OnlyCurrentThread,
                AllThreads,
            }

            pub trait TestPartResultReporterInterface {
                fn report_test_part_result(&self, result: TestPartResult);
            }

            pub fn main() {
                let test_results = Arc::new(Mutex::new(Vec::new()));
                let reporter = ScopedFakeTestPartResultReporter::new(test_results.clone());

                reporter.report_test_part_result(TestPartResult {
                    type_: Type::FatalFailure,
                    message: "A fatal error occurred.".into(),
                });
            }

            use std::sync::Once;
            static INIT: Once = Once::new();
            static mut UNIT_TEST_IMPL: Option<Mutex<UnitTestImpl>> = None;

            fn get_unit_test_impl() -> &'static Mutex<UnitTestImpl> {
                unsafe {
                    INIT.call_once(|| {
                        UNIT_TEST_IMPL = Some(Mutex::new(UnitTestImpl::new()));
                    });
                    UNIT_TEST_IMPL.as_ref().expect("UNIT_TEST_IMPL not initialized")
                }
            }

            struct UnitTestImpl {
                test_part_result_reporter_for_current_thread: Option<TestPartResultReporter>,
                global_test_part_result_reporter: Option<TestPartResultReporter>,
            }

            impl UnitTestImpl {
                fn new() -> Self {
                    UnitTestImpl {
                        test_part_result_reporter_for_current_thread: None,
                        global_test_part_result_reporter: None,
                    }
                }

                fn set_test_part_result_reporter_for_current_thread(&mut self, reporter: TestPartResultReporter) -> TestPartResultReporter {
                    let old_reporter = self.test_part_result_reporter_for_current_thread.take();
                    self.test_part_result_reporter_for_current_thread = Some(reporter);
                    old_reporter.unwrap_or_else(|| Arc::new(Mutex::new(ScopedFakeTestPartResultReporter::new(Arc::new(Mutex::new(Vec::new()))))))
                }

                fn set_global_test_part_result_reporter(&mut self, reporter: TestPartResultReporter) -> TestPartResultReporter {
                    let old_reporter = self.global_test_part_result_reporter.take();
                    self.global_test_part_result_reporter = Some(reporter);
                    old_reporter.unwrap_or_else(|| Arc::new(Mutex::new(ScopedFakeTestPartResultReporter::new(Arc::new(Mutex::new(Vec::new()))))))
                }
            }
        }
    }
}

fn main() {
    google_test_fused::gtest::gtest_spy::main();
}