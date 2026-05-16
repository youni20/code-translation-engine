use std::sync::{Arc, Mutex};

// Mocking types and traits
#[derive(Debug, Clone, PartialEq)]
enum TestPartResultType {
    Success,
    NonFatalFailure,
    FatalFailure,
}

#[derive(Debug, Clone)]
struct TestPartResult {
    type_: TestPartResultType,
    file_name: Option<String>,
    line_number: Option<u32>,
    message: String,
}

impl TestPartResult {
    fn fatally_failed(&self) -> bool {
        self.type_ == TestPartResultType::FatalFailure
    }

    fn nonfatally_failed(&self) -> bool {
        self.type_ == TestPartResultType::NonFatalFailure
    }
}

trait TestPartResultReporter {
    fn report_test_part_result(&self, result: TestPartResult);
}

#[derive(Debug, Clone)]
struct TestPartResultArray {
    array: Vec<TestPartResult>,
}

impl TestPartResultArray {
    fn new() -> Self {
        TestPartResultArray { array: Vec::new() }
    }

    fn append(&mut self, result: TestPartResult) {
        self.array.push(result)
    }

    fn size(&self) -> usize {
        self.array.len()
    }

    fn get_test_part_result(&self, index: usize) -> &TestPartResult {
        &self.array[index]
    }
}

// Mimicking C++ unique_ptr with Rust's ownership model
struct ScopedFakeTestPartResultReporter {
    intercept_mode: InterceptMode,
    result: Arc<Mutex<TestPartResultArray>>,
}

#[derive(Debug)]
enum InterceptMode {
    InterceptOnlyCurrentThread,
    InterceptAllThreads,
}

impl ScopedFakeTestPartResultReporter {
    fn new(mode: InterceptMode, result: Arc<Mutex<TestPartResultArray>>) -> Self {
        ScopedFakeTestPartResultReporter {
            intercept_mode: mode,
            result,
        }
    }

    fn init(&self) {
        // Initialize depending on intercept_mode
    }
}

impl TestPartResultReporter for ScopedFakeTestPartResultReporter {
    fn report_test_part_result(&self, result: TestPartResult) {
        let mut locked_result = self.result.lock().unwrap();
        locked_result.append(result);
    }
}

fn count_if<T, F>(c: &[T], predicate: F) -> usize
where
    F: Fn(&T) -> bool,
{
    c.iter().filter(|&x| predicate(x)).count()
}

fn for_each<T, F>(c: &[T], mut functor: F)
where
    F: FnMut(&T),
{
    for item in c.iter() {
        functor(item);
    }
}

fn main() {
    // Just a main function to illustrate usage and avoid warnings, as C++ specific
    // setup and testing environment can't be directly translated here. This acts as an entry point.
    let result_array = Arc::new(Mutex::new(TestPartResultArray::new()));
    let reporter = ScopedFakeTestPartResultReporter::new(InterceptMode::InterceptOnlyCurrentThread, result_array);

    let simulated_result = TestPartResult {
        type_: TestPartResultType::Success,
        file_name: Some("example.rs".into()),
        line_number: Some(42),
        message: "Test Passed".into(),
    };

    reporter.report_test_part_result(simulated_result.clone());

    let size = reporter.result.lock().unwrap().size();
    println!("Number of test results: {}", size);
}