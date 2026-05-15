use std::io::{self, Write};
use std::cmp::Ordering;
use std::fmt::{self, Display, Formatter};
use std::time::Instant;

// Constants
const DOCTEST_VERSION_MAJOR: u32 = 2;
const DOCTEST_VERSION_MINOR: u32 = 4;
const DOCTEST_VERSION_PATCH: u32 = 6;
const DOCTEST_VERSION_STR: &str = "2.4.6";
const DOCTEST_VERSION: u32 =
    DOCTEST_VERSION_MAJOR * 10000 + DOCTEST_VERSION_MINOR * 100 + DOCTEST_VERSION_PATCH;
const DOCTEST_MSVC: u32 = if cfg!(target_env = "msvc") { 1 } else { 0 };
const DOCTEST_CONFIG_USE_STD_HEADERS: bool = true;

// Structures
#[derive(Debug, PartialEq, Eq)]
struct Version {
    major: u32,
    minor: u32,
    patch: u32,
}

const DOCTEST_COMPILER: fn(usize) -> Version = |n| Version {
    major: (n / 10000) as u32,
    minor: ((n / 100) % 100) as u32,
    patch: (n % 100) as u32,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct SubcaseSignature {
    name: String,
    file: &'static str,
    line: usize,
}

impl PartialOrd for SubcaseSignature {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.line != other.line {
            return Some(self.line.cmp(&other.line));
        }
        if self.file != other.file {
            return Some(self.file.cmp(other.file));
        }
        Some(self.name.cmp(&other.name))
    }
}

impl Ord for SubcaseSignature {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum Color {
    None = 0,
    White,
    Red,
    Green,
    Blue,
    Cyan,
    Yellow,
    Grey,
    Bright = 0x10,
    BrightRed = Color::Bright as isize | Color::Red as isize,
    BrightGreen = Color::Bright as isize | Color::Green as isize,
    LightGrey = Color::Bright as isize | Color::Grey as isize,
    BrightWhite = Color::Bright as isize | Color::White as isize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StringWithData {
    data: Vec<u8>,
    heap_alloc: bool,
}

const MAX_INLINE_SIZE: usize = 23;

#[repr(C)]
struct View {
    ptr: *mut u8,
    size: u32,
    capacity: u32,
}

impl StringWithData {
    fn new() -> Self {
        Self {
            data: vec![0; std::mem::size_of::<View>()],
            heap_alloc: false,
        }
    }

    fn from_str(input: &str) -> Self {
        let size = input.len();
        match size <= MAX_INLINE_SIZE {
            true => {
                let mut data = vec![0; std::mem::size_of::<View>()];
                data[..size].copy_from_slice(input.as_bytes());
                data[MAX_INLINE_SIZE] = size as u8;
                Self {
                    data,
                    heap_alloc: false,
                }
            },
            false => {
                let mut data = vec![0; std::mem::size_of::<View>()];
                let mut buffer = input.as_bytes().to_vec();
                buffer.push(0);
                let view = View {
                    ptr: buffer.as_mut_ptr(),
                    size: size as u32,
                    capacity: (size + 1) as u32,
                };
                std::mem::forget(buffer);
                unsafe {
                    (data.as_mut_ptr() as *mut View).write(view);
                }
                Self {
                    data,
                    heap_alloc: true,
                }
            }
        }
    }

    fn size(&self) -> usize {
        match self.heap_alloc {
            true => unsafe { (*(self.data.as_ptr() as *const View)).size as usize },
            false => self.data[MAX_INLINE_SIZE] as usize,
        }
    }

    fn c_str(&self) -> &[u8] {
        match self.heap_alloc {
            true => unsafe { std::slice::from_raw_parts((*(self.data.as_ptr() as *const View)).ptr, self.size() + 1) },
            false => &self.data[..self.size() + 1],
        }
    }
}

impl Drop for StringWithData {
    fn drop(&mut self) {
        if self.heap_alloc {
            let ptr = self.data.as_ptr() as *const View;
            let view = unsafe { ptr.read() };
            unsafe { Vec::from_raw_parts(view.ptr, view.size as usize + 1, view.capacity as usize); }
        }
    }
}

impl Display for StringWithData {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        unsafe {
            writeln!(f, "{}", std::str::from_utf8_unchecked(self.c_str()))
        }
    }
}

impl From<&str> for StringWithData {
    fn from(s: &str) -> Self {
        Self::from_str(s)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum AssertType {
    Warn,
    Check,
    Require,
    WarnFalse,
    CheckFalse,
    RequireFalse,
    WarnThrows,
    CheckThrows,
    RequireThrows,
    WarnThrowsAs,
    CheckThrowsAs,
    RequireThrowsAs,
    WarnThrowsWith,
    CheckThrowsWith,
    RequireThrowsWith,
    WarnThrowsWithAs,
    CheckThrowsWithAs,
    RequireThrowsWithAs,
    WarnNoThrow,
    CheckNoThrow,
    RequireNoThrow,
    WarnEq,
    CheckEq,
    RequireEq,
    WarnNe,
    CheckNe,
    RequireNe,
    WarnGt,
    CheckGt,
    RequireGt,
    WarnLt,
    CheckLt,
    RequireLt,
    WarnGe,
    CheckGe,
    RequireGe,
    WarnLe,
    CheckLe,
    RequireLe,
    WarnUnary,
    CheckUnary,
    RequireUnary,
    WarnUnaryFalse,
    CheckUnaryFalse,
    RequireUnaryFalse,
}

impl Display for AssertType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            AssertType::Warn => write!(f, "WARN"),
            AssertType::Check => write!(f, "CHECK"),
            AssertType::Require => write!(f, "REQUIRE"),
            AssertType::WarnFalse => write!(f, "WARN_FALSE"),
            AssertType::CheckFalse => write!(f, "CHECK_FALSE"),
            AssertType::RequireFalse => write!(f, "REQUIRE_FALSE"),
            AssertType::WarnThrows => write!(f, "WARN_THROWS"),
            AssertType::CheckThrows => write!(f, "CHECK_THROWS"),
            AssertType::RequireThrows => write!(f, "REQUIRE_THROWS"),
            AssertType::WarnThrowsAs => write!(f, "WARN_THROWS_AS"),
            AssertType::CheckThrowsAs => write!(f, "CHECK_THROWS_AS"),
            AssertType::RequireThrowsAs => write!(f, "REQUIRE_THROWS_AS"),
            AssertType::WarnThrowsWith => write!(f, "WARN_THROWS_WITH"),
            AssertType::CheckThrowsWith => write!(f, "CHECK_THROWS_WITH"),
            AssertType::RequireThrowsWith => write!(f, "REQUIRE_THROWS_WITH"),
            AssertType::WarnThrowsWithAs => write!(f, "WARN_THROWS_WITH_AS"),
            AssertType::CheckThrowsWithAs => write!(f, "CHECK_THROWS_WITH_AS"),
            AssertType::RequireThrowsWithAs => write!(f, "REQUIRE_THROWS_WITH_AS"),
            AssertType::WarnNoThrow => write!(f, "WARN_NOTHROW"),
            AssertType::CheckNoThrow => write!(f, "CHECK_NOTHROW"),
            AssertType::RequireNoThrow => write!(f, "REQUIRE_NOTHROW"),
            AssertType::WarnEq => write!(f, "WARN_EQ"),
            AssertType::CheckEq => write!(f, "CHECK_EQ"),
            AssertType::RequireEq => write!(f, "REQUIRE_EQ"),
            AssertType::WarnNe => write!(f, "WARN_NE"),
            AssertType::CheckNe => write!(f, "CHECK_NE"),
            AssertType::RequireNe => write!(f, "REQUIRE_NE"),
            AssertType::WarnGt => write!(f, "WARN_GT"),
            AssertType::CheckGt => write!(f, "CHECK_GT"),
            AssertType::RequireGt => write!(f, "REQUIRE_GT"),
            AssertType::WarnLt => write!(f, "WARN_LT"),
            AssertType::CheckLt => write!(f, "CHECK_LT"),
            AssertType::RequireLt => write!(f, "REQUIRE_LT"),
            AssertType::WarnGe => write!(f, "WARN_GE"),
            AssertType::CheckGe => write!(f, "CHECK_GE"),
            AssertType::RequireGe => write!(f, "REQUIRE_GE"),
            AssertType::WarnLe => write!(f, "WARN_LE"),
            AssertType::CheckLe => write!(f, "CHECK_LE"),
            AssertType::RequireLe => write!(f, "REQUIRE_LE"),
            AssertType::WarnUnary => write!(f, "WARN_UNARY"),
            AssertType::CheckUnary => write!(f, "CHECK_UNARY"),
            AssertType::RequireUnary => write!(f, "REQUIRE_UNARY"),
            AssertType::WarnUnaryFalse => write!(f, "WARN_UNARY_FALSE"),
            AssertType::CheckUnaryFalse => write!(f, "CHECK_UNARY_FALSE"),
            AssertType::RequireUnaryFalse => write!(f, "REQUIRE_UNARY_FALSE"),
        }
    }
}

#[derive(Debug, Clone)]
struct AssertData {
    test_case: Option<String>,
    at: AssertType,
    file: &'static str,
    line: i32,
    expr: &'static str,
    failed: bool,
    threw: bool,
    exception: StringWithData,
    decomp: StringWithData,
    threw_as: bool,
    exception_type: &'static str,
    exception_string: &'static str,
}

#[derive(Debug, PartialEq, Clone)]
struct TestCaseData {
    file: StringWithData,
    line: usize,
    name: &'static str,
    test_suite: &'static str,
    description: &'static str,
    skip: bool,
    no_breaks: bool,
    no_output: bool,
    may_fail: bool,
    should_fail: bool,
    expected_failures: i32,
    timeout: f64,
}

#[derive(Debug, Clone)]
struct MessageData {
    message: StringWithData,
    file: &'static str,
    line: i32,
    severity: AssertType,
}

impl TestCaseData {
    fn new(
        file: String,
        line: usize,
        name: &'static str,
        test_suite: &'static str,
        description: &'static str,
        skip: bool,
        no_breaks: bool,
        no_output: bool,
        may_fail: bool,
        should_fail: bool,
        expected_failures: i32,
        timeout: f64,
    ) -> Self {
        TestCaseData {
            file: StringWithData::from(file.as_str()),
            line,
            name,
            test_suite,
            description,
            skip,
            no_breaks,
            no_output,
            may_fail,
            should_fail,
            expected_failures,
            timeout,
        }
    }
}

struct ContextOptions {
    cout: Box<dyn Write + Send + Sync>,
    cerr: Box<dyn Write + Send + Sync>,
    binary_name: StringWithData,
    current_test: Option<TestCaseData>,
    out: StringWithData,
    order_by: StringWithData,
    rand_seed: u32,
    first: Option<u32>,
    last: Option<u32>,
    abort_after: i32,
    subcase_filter_levels: i32,
    success: bool,
    case_sensitive: bool,
    exit: bool,
    duration: bool,
    no_throw: bool,
    no_exitcode: bool,
    no_run: bool,
    no_version: bool,
    no_colors: bool,
    force_colors: bool,
    no_breaks: bool,
    no_skip: bool,
    gnu_file_line: bool,
    no_path_in_filenames: bool,
    no_line_numbers: bool,
    no_debug_output: bool,
    no_skipped_summary: bool,
    no_time_in_output: bool,
    help: bool,
    version: bool,
    count: bool,
    list_test_cases: bool,
    list_test_suites: bool,
    list_reporters: bool,
}

struct TestFailureReasonEnum {}
impl TestFailureReasonEnum {
    const NONE: u32 = 0;
    const ASSERT_FAILURE: u32 = 1;
    const EXCEPTION: u32 = 2;
    const CRASH: u32 = 4;
    const TOO_MANY_FAILED_ASSERTS: u32 = 8;
    const TIMEOUT: u32 = 16;
    const SHOULD_HAVE_FAILED_BUT_DIDNT: u32 = 32;
    const SHOULD_HAVE_FAILED_AND_DID: u32 = 64;
    const DIDNT_FAIL_EXACTLY_NUM_TIMES: u32 = 128;
    const FAILED_EXACTLY_NUM_TIMES: u32 = 256;
    const COULD_HAVE_FAILED_AND_DID: u32 = 512;
}

pub fn string_to_color(input: &str) -> Option<Color> {
    match input {
        "None" => Some(Color::None),
        "White" => Some(Color::White),
        "Red" => Some(Color::Red),
        "Green" => Some(Color::Green),
        "Blue" => Some(Color::Blue),
        "Cyan" => Some(Color::Cyan),
        "Yellow" => Some(Color::Yellow),
        "Grey" => Some(Color::Grey),
        "LightGrey" => Some(Color::LightGrey),
        "BrightRed" => Some(Color::BrightRed),
        "BrightGreen" => Some(Color::BrightGreen),
        "BrightWhite" => Some(Color::BrightWhite),
        _ => None,
    }
}

#[derive(Debug)]
struct CurrentTestCaseStats {
    num_asserts: i32,
    num_asserts_failed: i32,
    seconds: f64,
    failure_flags: i32,
}

#[derive(Debug)]
struct TestCaseException {
    error_string: StringWithData,
    is_crash: bool,
}

#[derive(Debug)]
struct TestRunStats {
    num_test_cases: u32,
    num_test_cases_passing_filters: u32,
    num_test_suites_passing_filters: u32,
    num_test_cases_failed: u32,
    num_asserts: i32,
    num_asserts_failed: i32,
}

struct QueryData {
    run_stats: Option<TestRunStats>,
    data: Vec<TestCaseData>,
}

enum InOutTypeEnum {
    Stdout,
    Stderr,
    File,
}

enum TestType {
    A,
    B,
    C,
}

trait IReporter {
    fn report_query(&self, _data: QueryData);
    fn test_run_start(&self);
    fn test_run_end(&self, _stats: TestRunStats);
    fn test_case_start(&self, _data: &TestCaseData);
    fn test_case_reenter(&self, _data: &TestCaseData);
    fn test_case_end(&self, _stats: &CurrentTestCaseStats);
    fn test_case_exception(&self, _exception: &TestCaseException);
    fn subcase_start(&self, _signature: &SubcaseSignature);
    fn subcase_end(&self);
    fn log_assert(&self, _data: &AssertData);
    fn log_message(&self, _data: &MessageData);
    fn test_case_skipped(&self, _data: &TestCaseData);
}

impl IReporter for () {
    fn report_query(&self, _data: QueryData) {}
    fn test_run_start(&self) {}
    fn test_run_end(&self, _stats: TestRunStats) {}
    fn test_case_start(&self, _data: &TestCaseData) {}
    fn test_case_reenter(&self, _data: &TestCaseData) {}
    fn test_case_end(&self, _stats: &CurrentTestCaseStats) {}
    fn test_case_exception(&self, _exception: &TestCaseException) {}
    fn subcase_start(&self, _signature: &SubcaseSignature) {}
    fn subcase_end(&self) {}
    fn log_assert(&self, _data: &AssertData) {}
    fn log_message(&self, _data: &MessageData) {}
    fn test_case_skipped(&self, _data: &TestCaseData) {}
}

trait IContextScope {
    fn stringify(&self, output: &mut std::fmt::Formatter);
}

trait ContextScope: IContextScope {
    fn new() -> Self;
    fn drop(&mut self);
}

struct ContextScopeBase {}
impl IContextScope for ContextScopeBase {
    fn stringify(&self, _: &mut fmt::Formatter) {
        // no-op
    }
}

impl ContextScopeBase {
    fn drop(&mut self) {
        // no-op
    }
}

struct StringMakerBase {}
impl StringMakerBase {
    fn convert<T: Display>(&self, input: T) -> StringWithData {
        StringWithData::from(format!("{}", input).as_str())
    }
}

struct BinaryExpr<L, R> {
    lhs: L,
    rhs: R,
    op: &'static str,
    decomp: StringWithData,
}

impl<L, R> BinaryExpr<L, R>
where
    L: Display,
    R: Display,
{
    fn stringify(&self) -> StringWithData {
        StringWithData::from(format!("{}{}{}", self.lhs, self.op, self.rhs).as_str())
    }
}

trait DeferredFalse {
    fn value() -> bool;
}

struct AssertActionEnum;
impl AssertActionEnum {
    const NOTHING: i32 = 0;
    const DGBREAK: i32 = 1;
    const SHOULDTHROW: i32 = 2;
}

pub fn my_memcpy<T>(dest: &mut T, src: &T, num: usize) {
    let p_dest: *mut u8 = dest as *mut _ as *mut u8;
    let p_src: *const u8 = src as *const _ as *const u8;
    unsafe {
        std::ptr::copy_nonoverlapping(p_src, p_dest, num);
    }
}

#[derive(Default)]
struct Timer {
    start_time: Option<Instant>,
}

impl Timer {
    fn start(&mut self) {
        self.start_time = Some(Instant::now());
    }

    fn elapsed_microseconds(&self) -> Option<u64> {
        self.start_time.map(|start_time| start_time.elapsed().as_micros() as u64)
    }

    fn elapsed_seconds(&self) -> Option<f64> {
        self.start_time.map(|start_time| start_time.elapsed().as_secs_f64())
    }
}

pub mod assert_type {
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub enum Enum {
        //_special
        IsWarn = 1,
        IsCheck = 2 * (Self::IsWarn as isize),
        IsRequire = 2 * (Self::IsCheck as isize),
        IsNormal = 2 * (Self::IsRequire as isize),
        IsThrows = 2 * (Self::IsNormal as isize),
        IsThrowsAs = 2 * (Self::IsThrows as isize),
        IsThrowsWith = 2 * (Self::IsThrowsAs as isize),
        IsNothrow = 2 * (Self::IsThrowsWith as isize),
        IsFalse = 2 * (Self::IsNothrow as isize),
        IsUnary = 2 * (Self::IsFalse as isize),
        IsEq = 2 * (Self::IsUnary as isize),
        IsNe = 2 * (Self::IsEq as isize),
        IsLt = 2 * (Self::IsNe as isize),
        IsGt = 2 * (Self::IsLt as isize),
        IsGe = 2 * (Self::IsGt as isize),
        IsLe = 2 * (Self::IsGe as isize),
        DTWarn = (Self::IsNormal as isize) | (Self::IsWarn as isize),
        DTCheck = (Self::IsNormal as isize) | (Self::IsCheck as isize),
        DTRequire = (Self::IsNormal as isize) | (Self::IsRequire as isize),
        DTWarnFalse = (Self::IsNormal as isize) | (Self::IsFalse as isize) | (Self::IsWarn as isize),
        DTCheckFalse = (Self::IsNormal as isize) | (Self::IsFalse as isize) | (Self::IsCheck as isize),
        DTRequireFalse = (Self::IsNormal as isize) | (Self::IsFalse as isize) | (Self::IsRequire as isize),
        DTWarnThrows = (Self::IsThrows as isize) | (Self::IsWarn as isize),
        DTCheckThrows = (Self::IsThrows as isize) | (Self::IsCheck as isize),
        DTRequireThrows = (Self::IsThrows as isize) | (Self::IsRequire as isize),
        DTWarnThrowsAs = (Self::IsThrowsAs as isize) | (Self::IsWarn as isize),
        DTCheckThrowsAs = (Self::IsThrowsAs as isize) | (Self::IsCheck as isize),
        DTRequireThrowsAs = (Self::IsThrowsAs as isize) | (Self::IsRequire as isize),
        DTWarnThrowsWith = (Self::IsThrowsWith as isize) | (Self::IsWarn as isize),
        DTCheckThrowsWith = (Self::IsThrowsWith as isize) | (Self::IsCheck as isize),
        DTRequireThrowsWith = (Self::IsThrowsWith as isize) | (Self::IsRequire as isize),
        DTWarnThrowsWithAs = (Self::IsThrowsWith as isize) | (Self::IsThrowsAs as isize) | (Self::IsWarn as isize),
        DTCheckThrowsWithAs = (Self::IsThrowsWith as isize) | (Self::IsThrowsAs as isize) | (Self::IsCheck as isize),
        DTRequireThrowsWithAs = (Self::IsThrowsWith as isize) | (Self::IsThrowsAs as isize) | (Self::IsRequire as isize),
        DTWarnNothrow = (Self::IsNothrow as isize) | (Self::IsWarn as isize),
        DTCheckNothrow = (Self::IsNothrow as isize) | (Self::IsCheck as isize),
        DTRequireNothrow = (Self::IsNothrow as isize) | (Self::IsRequire as isize),
        DTWarnEq = (Self::IsNormal as isize) | (Self::IsEq as isize) | (Self::IsWarn as isize),
        DTCheckEq = (Self::IsNormal as isize) | (Self::IsEq as isize) | (Self::IsCheck as isize),
        DTRequireEq = (Self::IsNormal as isize) | (Self::IsEq as isize) | (Self::IsRequire as isize),
        DTWarnNe = (Self::IsNormal as isize) | (Self::IsNe as isize) | (Self::IsWarn as isize),
        DTCheckNe = (Self::IsNormal as isize) | (Self::IsNe as isize) | (Self::IsCheck as isize),
        DTRequireNe = (Self::IsNormal as isize) | (Self::IsNe as isize) | (Self::IsRequire as isize),
        DTWarnGt = (Self::IsNormal as isize) | (Self::IsGt as isize) | (Self::IsWarn as isize),
        DTCheckGt = (Self::IsNormal as isize) | (Self::IsGt as isize) | (Self::IsCheck as isize),
        DTRequireGt = (Self::IsNormal as isize) | (Self::IsGt as isize) | (Self::IsRequire as isize),
        DTWarnLt = (Self::IsNormal as isize) | (Self::IsLt as isize) | (Self::IsWarn as isize),
        DTCheckLt = (Self::IsNormal as isize) | (Self::IsLt as isize) | (Self::IsCheck as isize),
        DTRequireLt = (Self::IsNormal as isize) | (Self::IsLt as isize) | (Self::IsRequire as isize),
        DTWarnGe = (Self::IsNormal as isize) | (Self::IsGe as isize) | (Self::IsWarn as isize),
        DTCheckGe = (Self::IsNormal as isize) | (Self::IsGe as isize) | (Self::IsCheck as isize),
        DTRequireGe = (Self::IsNormal as isize) | (Self::IsGe as isize) | (Self::IsRequire as isize),
        DTWarnLe = (Self::IsNormal as isize) | (Self::IsLe as isize) | (Self::IsWarn as isize),
        DTCheckLe = (Self::IsNormal as isize) | (Self::IsLe as isize) | (Self::IsCheck as isize),
        DTRequireLe = (Self::IsNormal as isize) | (Self::IsLe as isize) | (Self::IsRequire as isize),
        DTWarnUnary = (Self::IsNormal as isize) | (Self::IsUnary as isize) | (Self::IsWarn as isize),
        DTCheckUnary = (Self::IsNormal as isize) | (Self::IsUnary as isize) | (Self::IsCheck as isize),
        DTRequireUnary = (Self::IsNormal as isize) | (Self::IsUnary as isize) | (Self::IsRequire as isize),
        DTWarnUnaryFalse = (Self::IsNormal as isize) | (Self::IsFalse as isize) | (Self::IsUnary as isize) | (Self::IsWarn as isize),
        DTCheckUnaryFalse = (Self::IsNormal as isize) | (Self::IsFalse as isize) | (Self::IsUnary as isize) | (Self::IsCheck as isize),
        DTRequireUnaryFalse = (Self::IsNormal as isize) | (Self::IsFalse as isize) | (Self::IsUnary as isize) | (Self::IsRequire as isize),
    }
}

fn main() {}