// Implementing the ScopedFakeTestPartResultReporter
struct ScopedFakeTestPartResultReporter<'a> {
    intercept_mode: InterceptMode,
    result: &'a mut TestPartResultArray,
    old_reporter: Option<Box<dyn TestPartResultReporterInterface + 'a>>,
}

impl<'a> ScopedFakeTestPartResultReporter<'a> {
    fn new(intercept_mode: InterceptMode, result: &'a mut TestPartResultArray) -> Self {
        let impl_instance = get_unit_test_impl();
        let old_reporter = match intercept_mode {
            InterceptMode::InterceptOnlyCurrentThread => {
                impl_instance.get_test_part_result_reporter_for_current_thread().map(|r| Box::new(r))
            }
            InterceptMode::InterceptAllThreads => {
                impl_instance.get_global_test_part_result_reporter().map(|r| Box::new(r))
            }
        };

        let mut reporter = ScopedFakeTestPartResultReporter {
            intercept_mode,
            result,
            old_reporter: old_reporter,
        };

        match reporter.intercept_mode {
            InterceptMode::InterceptAllThreads => {
                impl_instance.set_global_test_part_result_reporter(Some(&mut reporter))
            }
            InterceptMode::InterceptOnlyCurrentThread => {
                impl_instance.set_test_part_result_reporter_for_current_thread(Some(&mut reporter))
            }
        }

        reporter
    }
}

impl<'a> Drop for ScopedFakeTestPartResultReporter<'a> {
    fn drop(&mut self) {
        let impl_instance = get_unit_test_impl();
        match self.intercept_mode {
            InterceptMode::InterceptAllThreads => {
                let old_reporter = self.old_reporter.take();
                impl_instance.set_global_test_part_result_reporter(old_reporter.map(|r| Box::leak(r) as &mut _))
            }
            InterceptMode::InterceptOnlyCurrentThread => {
                let old_reporter = self.old_reporter.take();
                impl_instance.set_test_part_result_reporter_for_current_thread(old_reporter.map(|r| Box::leak(r) as &mut _))
            }
        }
    }
}

impl TestPartResultReporterInterface for ScopedFakeTestPartResultReporter<'_> {
    fn report_test_part_result(&mut self, result: &TestPartResult) {
        self.result.append(result);
    }
}

// Implement the TestPartResultArray
struct TestPartResultArray {
    results: Vec<TestPartResult>,
}

impl TestPartResultArray {
    fn append(&mut self, result: &TestPartResult) {
        self.results.push(result.clone());
    }
}

// TestPartResult and other related enums/structs definitions and code
#[derive(Clone)]
struct TestPartResult;

enum InterceptMode {
    InterceptOnlyCurrentThread,
    InterceptAllThreads,
}

trait TestPartResultReporterInterface {
    fn report_test_part_result(&mut self, result: &TestPartResult);
}

fn get_unit_test_impl() -> &'static mut UnitTestImpl {
    // Mock implementation: returns a mutable reference to some UnitTestImpl
    unimplemented!()
}

struct UnitTestImpl;

impl UnitTestImpl {
    fn get_global_test_part_result_reporter(
        &mut self,
    ) -> Option<&mut dyn TestPartResultReporterInterface> {
        // Mock implementation
        unimplemented!()
    }

    fn get_test_part_result_reporter_for_current_thread(
        &mut self,
    ) -> Option<&mut dyn TestPartResultReporterInterface> {
        // Mock implementation
        unimplemented!()
    }

    fn set_global_test_part_result_reporter(
        &mut self,
        _reporter: Option<&mut dyn TestPartResultReporterInterface>,
    ) {
        // Mock implementation
        unimplemented!()
    }

    fn set_test_part_result_reporter_for_current_thread(
        &mut self,
        _reporter: Option<&mut dyn TestPartResultReporterInterface>,
    ) {
        // Mock implementation
        unimplemented!()
    }
}

fn main() {
    // Entry point for the program
}