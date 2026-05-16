// ======================================================================
//
// doctest.rs - the lightest feature-rich Rust single-header testing framework for unit tests and TDD
//
// Copyright (c) 2016-2021 Viktor Kirilov
//
// Distributed under the MIT Software License
// See accompanying file LICENSE.txt or copy at
// https://opensource.org/licenses/MIT
//
// The documentation can be found at the library's page:
// https://github.com/onqtam/doctest/blob/master/doc/markdown/readme.md
//
// =================================================================================================
// =================================================================================================
// =================================================================================================
//
// The library is heavily influenced by Catch - https://github.com/catchorg/Catch2
// which uses the Boost Software License - Version 1.0
// see here - https://github.com/catchorg/Catch2/blob/master/LICENSE.txt
//
// The concept of subcases (sections in Catch) and expression decomposition are from there.
// Some parts of the code are taken directly:
// - stringification - the detection of "ostream& operator<<(ostream&, const T&)" and StringMaker<>
// - the Approx() helper class for floating point comparison
// - colors in the console
// - breaking into a debugger
// - signal / SEH handling
// - timer
// - XmlWriter class - thanks to Phil Nash for allowing the direct reuse (AKA copy/paste)
//
// The expression decomposing templates are taken from lest - https://github.com/martinmoene/lest
// which uses the Boost Software License - Version 1.0
// see here - https://github.com/martinmoene/lest/blob/master/LICENSE.txt
//
// =================================================================================================
// =================================================================================================
// =================================================================================================

use std::any::Any;
use std::fmt::{self, Debug, Display};
use std::io::{self, Write};
use std::result::Result as StdResult;
use std::time::{Duration, Instant};

// =================================================================================================
// == VERSION ======================================================================================
// =================================================================================================

const DOCTEST_VERSION_MAJOR: u32 = 2;
const DOCTEST_VERSION_MINOR: u32 = 4;
const DOCTEST_VERSION_PATCH: u32 = 6;
const DOCTEST_VERSION_STR: &str = "2.4.6";

const DOCTEST_VERSION: u32 = DOCTEST_VERSION_MAJOR * 10000 + DOCTEST_VERSION_MINOR * 100 + DOCTEST_VERSION_PATCH;

// =================================================================================================
// == COMPILER VERSION =============================================================================
// =================================================================================================

// Ideas for the version stuff are taken from here: https://github.com/cxxstuff/cxx_detect

const fn doctest_compiler(major: u32, minor: u32, patch: u32) -> u32 {
    major * 10000000 + minor * 100000 + patch
}

// =================================================================================================
// == COMPILER WARNINGS HELPERS ====================================================================
// =================================================================================================

// TODO: Implement compiler warnings handling if needed

// =================================================================================================
// == COMPILER WARNINGS ============================================================================
// =================================================================================================

// TODO: Implement compiler warnings suppression if needed

// =================================================================================================
// == FEATURE DETECTION ============================================================================
// =================================================================================================

// General compiler feature support table: https://en.cppreference.com/w/cpp/compiler_support
// MSVC C++11 feature support table: https://msdn.microsoft.com/en-us/library/hh567368.aspx
// GCC C++11 feature support table: https://gcc.gnu.org/projects/cxx-status.html
// MSVC version table:
// https://en.wikipedia.org/wiki/Microsoft_Visual_C%2B%2B#Internal_version_numbering
// MSVC++ 14.2 (16) _MSC_VER == 1920 (Visual Studio 2019)
// MSVC++ 14.1 (15) _MSC_VER == 1910 (Visual Studio 2017)
// MSVC++ 14.0      _MSC_VER == 1900 (Visual Studio 2015)
// MSVC++ 12.0      _MSC_VER == 1800 (Visual Studio 2013)
// MSVC++ 11.0      _MSC_VER == 1700 (Visual Studio 2012)
// MSVC++ 10.0      _MSC_VER == 1600 (Visual Studio 2010)
// MSVC++ 9.0       _MSC_VER == 1500 (Visual Studio 2008)
// MSVC++ 8.0       _MSC_VER == 1400 (Visual Studio 2005)

// =================================================================================================
// == FEATURE DETECTION END ========================================================================
// =================================================================================================

// Internal macros for string concatenation and anonymous variable name generation

macro_rules! doctest_cat_impl {
    ($s1:expr, $s2:expr) => {
        concat!($s1, $s2)
    };
}

macro_rules! doctest_cat {
    ($s1:expr, $s2:expr) => {
        doctest_cat_impl!($s1, $s2)
    };
}

macro_rules! doctest_anonymous {
    ($x:expr) => {
        doctest_cat!($x, line!())
    };
}

// =================================================================================================
// == MAIN FUNCTIONALITY ===========================================================================
// =================================================================================================

struct TestCase {
    name: &'static str,
    suite: &'static str,
    func: fn(),
}

impl TestCase {
    pub fn new(name: &'static str, suite: &'static str, func: fn()) -> Self {
        TestCase { name, suite, func }
    }
}

struct TestSuite {
    name: &'static str,
    description: Option<&'static str>,
    test_cases: Vec<TestCase>,
}

impl TestSuite {
    pub fn new(name: &'static str) -> Self {
        TestSuite {
            name,
            description: None,
            test_cases: vec![],
        }
    }

    pub fn add_test_case(&mut self, name: &'static str, func: fn()) {
        self.test_cases.push(TestCase::new(name, self.name, func));
    }

    pub fn run(&self) {
        println!("Running test suite: {}", self.name);
        if let Some(desc) = self.description {
            println!("Description: {}", desc);
        }
        for test in &self.test_cases {
            println!("Running test case: {}/{}", test.suite, test.name);
            (test.func)();
        }
    }
}

pub struct Context {
    test_suites: Vec<TestSuite>,
}

impl Context {
    pub fn new() -> Self {
        Context {
            test_suites: vec![],
        }
    }

    pub fn add_test_suite(&mut self, suite: TestSuite) {
        self.test_suites.push(suite);
    }

    pub fn run(&self) {
        for suite in &self.test_suites {
            suite.run();
        }
    }
}

fn main() {
    // Example usage
    let mut context = Context::new();
    
    let mut suite1 = TestSuite::new("Example Suite");
    suite1.add_test_case("Test Case 1", || { println!("Executing Test Case 1"); });
    suite1.add_test_case("Test Case 2", || { println!("Executing Test Case 2"); });
    context.add_test_suite(suite1);
    
    context.run();
}