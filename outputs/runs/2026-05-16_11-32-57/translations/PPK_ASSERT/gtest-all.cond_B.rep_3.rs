use std::sync::Mutex;

pub struct TestPartResultArray {
    array: Vec<TestPartResult>,
}

impl TestPartResultArray {
    pub fn new() -> Self {
        TestPartResultArray { array: Vec::new() }
    }

    pub fn append(&mut self, result: TestPartResult) {
        self.array.push(result);
    }

    pub fn get_test_part_result(&self, index: usize) -> &TestPartResult {
        if index >= self.size() {
            panic!("Invalid index ({}) into TestPartResultArray.", index);
        }
        &self.array[index]
    }

    pub fn size(&self) -> usize {
        self.array.len()
    }
}

pub struct TestPartResult {
    file_name: String,
    line_number: i32,
    message: String,
    result_type: TestPartResultType,
}

impl TestPartResult {
    pub fn new(file_name: &str, line_number: i32, message: &str, result_type: TestPartResultType) -> Self {
        TestPartResult {
            file_name: file_name.to_string(),
            line_number,
            message: message.to_string(),
            result_type,
        }
    }

    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    pub fn line_number(&self) -> i32 {
        self.line_number
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn result_type(&self) -> TestPartResultType {
        self.result_type
    }
}

#[derive(Clone, Copy)]
pub enum TestPartResultType {
    Success,
    NonFatalFailure,
    FatalFailure,
}

pub struct ScopedFakeTestPartResultReporter<'a> {
    result: &'a mut TestPartResultArray,
}

impl<'a> ScopedFakeTestPartResultReporter<'a> {
    pub fn new(result: &'a mut TestPartResultArray) -> Self {
        ScopedFakeTestPartResultReporter { result }
    }

    pub fn report_test_part_result(&mut self, result: TestPartResult) {
        self.result.append(result);
    }
}

pub struct UnitTest;

impl UnitTest {
    pub fn get_instance() -> &'static Self {
        static INSTANCE: UnitTest = UnitTest;
        &INSTANCE
    }

    pub fn add_test_part_result(
        &self,
        result_type: TestPartResultType,
        file_name: &str,
        line_number: i32,
        message: &str,
    ) {
        let mut gtest_failures = test_results().lock().unwrap();
        let result = TestPartResult::new(file_name, line_number, message, result_type);
        gtest_failures.append(result);
    }
}

use std::sync::Once;
static INIT: Once = Once::new();
static mut TEST_RESULTS: Option<Mutex<TestPartResultArray>> = None;

fn test_results() -> &'static Mutex<TestPartResultArray> {
    unsafe {
        INIT.call_once(|| {
            TEST_RESULTS = Some(Mutex::new(TestPartResultArray::new()));
        });
        TEST_RESULTS.as_ref().expect("TEST_RESULTS is not initialized")
    }
}

fn main() {}