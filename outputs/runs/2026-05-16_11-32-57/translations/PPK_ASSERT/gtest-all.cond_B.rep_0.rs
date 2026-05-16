#![allow(non_snake_case)]

pub struct ScopedFakeTestPartResultReporter<'a> {
    intercept_mode_: InterceptMode,
    old_reporter_: Box<dyn TestPartResultReporterInterface>,
    result_: &'a mut TestPartResultArray,
}

impl<'a> ScopedFakeTestPartResultReporter<'a> {
    pub fn new(result: &'a mut TestPartResultArray) -> ScopedFakeTestPartResultReporter<'a> {
        Self::shared_ctor(result, InterceptMode::INTERCEPT_ONLY_CURRENT_THREAD)
    }

    pub fn shared_ctor(
        result: &'a mut TestPartResultArray,
        intercept_mode: InterceptMode,
    ) -> ScopedFakeTestPartResultReporter<'a> {
        let impl_ref = GetUnitTestImpl();
        let old_reporter = if intercept_mode == InterceptMode::INTERCEPT_ALL_THREADS {
            impl_ref.set_global_test_part_result_reporter(Box::new(DummyReporter));
        } else {
            impl_ref.set_test_part_result_reporter_for_current_thread(Box::new(DummyReporter));
        };
        ScopedFakeTestPartResultReporter {
            intercept_mode_: intercept_mode,
            old_reporter_: Box::new(DummyReporter), // DummyReporter as placeholder
            result_: result,
        }
    }

    pub fn report_test_part_result(&mut self, result: TestPartResult) {
        self.result_.append(result);
    }
}

impl<'a> Drop for ScopedFakeTestPartResultReporter<'a> {
    fn drop(&mut self) {
        let impl_ref = GetUnitTestImpl();
        if self.intercept_mode_ == InterceptMode::INTERCEPT_ALL_THREADS {
            impl_ref.set_global_test_part_result_reporter(Box::new(DummyReporter));
        } else {
            impl_ref.set_test_part_result_reporter_for_current_thread(Box::new(DummyReporter));
        }
    }
}

pub struct SingleFailureChecker<'a> {
    results_: &'a TestPartResultArray,
    type_: TestPartResultType,
    substr_: String,
}

impl<'a> SingleFailureChecker<'a> {
    pub fn new(
        results: &'a TestPartResultArray,
        type_: TestPartResultType,
        substr: String,
    ) -> SingleFailureChecker<'a> {
        SingleFailureChecker {
            results_: results,
            type_: type_,
            substr_: substr,
        }
    }
}

impl<'a> Drop for SingleFailureChecker<'a> {
    fn drop(&mut self) {
        assert!(has_one_failure(self.results_, self.type_.clone(), &self.substr_));
    }
}

pub struct TestPartResultArray(Vec<TestPartResult>);

impl TestPartResultArray {
    pub fn append(&mut self, result: TestPartResult) {
        self.0.push(result);
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum InterceptMode {
    INTERCEPT_ONLY_CURRENT_THREAD,
    INTERCEPT_ALL_THREADS,
}

pub trait TestPartResultReporterInterface {
    fn ReportTestPartResult(&mut self, result: TestPartResult);
}

#[derive(PartialEq)]
pub enum TestPartResultType {
    SUCCESS,
    NONFATAL_FAILURE,
    FATAL_FAILURE,
}

impl Clone for TestPartResultType {
    fn clone(&self) -> Self {
        match self {
            TestPartResultType::SUCCESS => TestPartResultType::SUCCESS,
            TestPartResultType::NONFATAL_FAILURE => TestPartResultType::NONFATAL_FAILURE,
            TestPartResultType::FATAL_FAILURE => TestPartResultType::FATAL_FAILURE,
        }
    }
}

pub struct TestPartResult {
    type_: TestPartResultType,
    message_: String,
}

pub struct UnitTestImpl {
    global_test_part_result_reporter_: Box<dyn TestPartResultReporterInterface>,
    per_thread_test_part_result_reporter_: Box<dyn TestPartResultReporterInterface>,
}

impl UnitTestImpl {
    pub fn set_global_test_part_result_reporter(
        &mut self,
        reporter: Box<dyn TestPartResultReporterInterface>,
    ) {
        self.global_test_part_result_reporter_ = reporter;
    }

    pub fn set_test_part_result_reporter_for_current_thread(
        &mut self,
        reporter: Box<dyn TestPartResultReporterInterface>,
    ) {
        self.per_thread_test_part_result_reporter_ = reporter;
    }
}

fn GetUnitTestImpl() -> &'static mut UnitTestImpl {
    // Assume UnitTestImpl is a singleton
    #![allow(dead_code)]
    todo!()
}

fn has_one_failure(
    results: &TestPartResultArray,
    type_: TestPartResultType,
    substr: &str,
) -> bool {
    let expected = if type_ == TestPartResultType::FATAL_FAILURE {
        "1 fatal failure"
    } else {
        "1 non-fatal failure"
    };
    if results.0.len() != 1 {
        return false;
    }
    let r = &results.0[0];
    r.type_ == type_ && r.message_.contains(substr)
}

pub struct DummyReporter;

impl TestPartResultReporterInterface for DummyReporter {
    fn ReportTestPartResult(&mut self, _result: TestPartResult) {}
}

fn main() {
    // Simulation placeholder.
}