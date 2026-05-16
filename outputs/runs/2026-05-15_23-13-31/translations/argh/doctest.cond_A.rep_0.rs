use std::collections::{HashMap, HashSet};
use std::fmt::{self, Display, Formatter};
use std::time::{Duration};

pub struct DoctestFramework {
    version: (u32, u32, u32),
    context_options: ContextOptions,
    registered_tests: HashMap<String, TestCase>,
    reporters: HashMap<String, Box<dyn Reporter>>,
    listeners: HashMap<String, Box<dyn Reporter>>,
}

impl DoctestFramework {
    pub fn new() -> DoctestFramework {
        DoctestFramework {
            version: (2, 4, 6),
            context_options: ContextOptions::default(),
            registered_tests: HashMap::new(),
            reporters: HashMap::new(),
            listeners: HashMap::new(),
        }
    }

    pub fn register_test_case(
        &mut self,
        name: &str,
        test_case: TestCase,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.registered_tests.contains_key(name) {
            return Err(Box::new(TestCaseRegistrationError(format!(
                "Test case '{}' is already registered.",
                name
            ))));
        }
        self.registered_tests.insert(name.to_string(), test_case);
        Ok(())
    }

    pub fn register_reporter(
        &mut self,
        name: &str,
        reporter: Box<dyn Reporter>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.reporters.contains_key(name) {
            return Err(Box::new(ReporterRegistrationError(format!(
                "Reporter '{}' is already registered.",
                name
            ))));
        }
        self.reporters.insert(name.to_string(), reporter);
        Ok(())
    }

    pub fn register_listener(
        &mut self,
        name: &str,
        listener: Box<dyn Reporter>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.listeners.contains_key(name) {
            return Err(Box::new(ListenerRegistrationError(format!(
                "Listener '{}' is already registered.",
                name
            ))));
        }
        self.listeners.insert(name.to_string(), listener);
        Ok(())
    }

    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Filtering and running logic
        Ok(())
    }
}

#[derive(Default, Debug)]
struct ContextOptions {
    filters: Vec<HashSet<String>>,
    count: bool,
    no_run: bool,
    list_test_cases: bool,
    list_test_suites: bool,
    list_reporters: bool,
    exit: bool,
    version: bool,
    help: bool,
    abort_after: usize,
    no_colors: bool,
    no_exitcode: bool,
    no_path_in_filenames: bool,
}

struct TestCase {
    file: String,
    line: usize,
    name: String,
    description: Option<String>,
    test_suite: Option<String>,
    skip: bool,
    no_breaks: bool,
    no_output: bool,
    may_fail: bool,
    should_fail: bool,
    expected_failures: usize,
    timeout: Duration,
    func: Box<dyn Fn() -> Result<(), Box<dyn std::error::Error>>>,
}

impl std::fmt::Debug for TestCase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TestCase")
            .field("file", &self.file)
            .field("line", &self.line)
            .field("name", &self.name)
            .field("description", &self.description)
            .field("test_suite", &self.test_suite)
            .field("skip", &self.skip)
            .field("no_breaks", &self.no_breaks)
            .field("no_output", &self.no_output)
            .field("may_fail", &self.may_fail)
            .field("should_fail", &self.should_fail)
            .field("expected_failures", &self.expected_failures)
            .field("timeout", &self.timeout)
            .finish()
    }
}

#[derive(Default)]
struct TestRunStats {
    num_test_cases: usize,
    num_test_cases_passing_filters: usize,
    num_test_suites_passing_filters: usize,
    num_test_cases_failed: usize,
    num_asserts: usize,
    num_asserts_failed: usize,
}

struct AssertData {
    test_case: Option<TestCase>,
    at: AssertType,
    file: String,
    line: usize,
    expr: String,
    failed: bool,
    threw: bool,
    exception: Option<String>,
    decomp: String,
    threw_as: bool,
    exception_type: Option<String>,
    exception_string: Option<String>,
}

pub enum AssertType {
    IsWarn,
    IsCheck,
    IsRequire,
    IsNormal,
    IsThrows,
    IsThrowsAs,
    IsThrowsWith,
    IsNothrow,
    IsFalse,
    IsUnary,
    IsEq,
    IsNe,
    IsLt,
    IsGt,
    IsGe,
    IsLe,
}

impl Display for AssertType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                AssertType::IsWarn => "WARN",
                AssertType::IsCheck => "CHECK",
                AssertType::IsRequire => "REQUIRE",
                AssertType::IsFalse => "FALSE",
                _ => "UNKNOWN",
            }
        )
    }
}

impl Display for AssertData {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "Assert Data: Test Case: {:?}, Type: {}, File: {}, Line: {}, Expr: {}, Failed: {}",
            self.test_case,
            self.at,
            self.file,
            self.line,
            self.expr,
            self.failed
        )
    }
}

trait Reporter {
    fn report(&self, data: AssertData);
}

struct TestCaseRegistrationError(String);
impl std::fmt::Debug for TestCaseRegistrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TestCaseRegistrationError: {}", self.0)
    }
}
impl std::fmt::Display for TestCaseRegistrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TestCaseRegistrationError: {}", self.0)
    }
}
impl std::error::Error for TestCaseRegistrationError {}

struct ReporterRegistrationError(String);
impl std::fmt::Debug for ReporterRegistrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ReporterRegistrationError: {}", self.0)
    }
}
impl std::fmt::Display for ReporterRegistrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ReporterRegistrationError: {}", self.0)
    }
}
impl std::error::Error for ReporterRegistrationError {}

struct ListenerRegistrationError(String);
impl std::fmt::Debug for ListenerRegistrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ListenerRegistrationError: {}", self.0)
    }
}
impl std::fmt::Display for ListenerRegistrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ListenerRegistrationError: {}", self.0)
    }
}
impl std::error::Error for ListenerRegistrationError {}

fn main() {
    // Instantiate the doctest framework
    let mut doctest = DoctestFramework::new();

    // Example of registering a test case
    doctest
        .register_test_case(
            "test_example",
            TestCase {
                file: String::from("example.rs"),
                line: 42,
                name: String::from("test_example"),
                description: Some(String::from("This is a test.")),
                test_suite: Some(String::from("Example Suite")),
                skip: false,
                no_breaks: false,
                no_output: false,
                may_fail: false,
                should_fail: false,
                expected_failures: 0,
                timeout: Duration::from_secs(1),
                func: Box::new(|| {
                    println!("Running example test case...");
                    Ok(())
                }),
            },
        )
        .unwrap();

    // Run the tests
    doctest.run().unwrap();
}