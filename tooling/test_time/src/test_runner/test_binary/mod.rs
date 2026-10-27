mod listed_test;
mod test_argument;

use std::path::PathBuf;
use std::time::Instant;

use listed_test::ListedTest;
use test_argument::TestArgument;

use crate::test_run::TestRun;
use crate::test_timing::TestTiming;
use task::Failure;
use task::{CommandLine, Program, Stdout};

#[derive(Debug)]
pub(super) struct TestBinary {
    path: PathBuf,
}

impl TestBinary {
    pub(super) fn new(path: PathBuf) -> TestBinary {
        TestBinary { path }
    }

    pub(super) fn runs(&self) -> Vec<TestRun> {
        self.tests().map_or_else(
            |failure| {
                eprintln!("{failure}");
                vec![TestRun::Unlisted(self.path.clone())]
            },
            |tests| tests.into_iter().map(|test| self.timed(test)).collect(),
        )
    }

    fn tests(&self) -> Result<Vec<ListedTest>, Failure> {
        let listed = self
            .line()
            .argument(TestArgument::List)
            .argument(TestArgument::Format)
            .argument(TestArgument::Terse)
            .captured::<Stdout>()?;
        Ok(listed
            .lines()
            .filter_map(|line| line.parse::<ListedTest>().ok())
            .collect())
    }

    fn timed(&self, test: ListedTest) -> TestRun {
        let started = Instant::now();
        let ran = self
            .line()
            .argument(TestArgument::Exact)
            .argument(&test)
            .argument(TestArgument::Quiet)
            .captured::<Stdout>();
        let timing = TestTiming::new(&self.path, test, started.elapsed());
        let outcome: fn(TestTiming) -> TestRun = ran
            .inspect_err(|failure| eprintln!("{failure}"))
            .map_or(TestRun::Failed, |_| TestRun::Passed);
        outcome(timing)
    }

    fn line(&self) -> CommandLine {
        CommandLine::new(Program::Executable(self.path.clone()))
    }
}
