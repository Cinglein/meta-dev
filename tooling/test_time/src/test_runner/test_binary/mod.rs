mod listed_test;
mod test_argument;

use std::fmt;
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
    arguments: Vec<String>,
}

impl TestBinary {
    const WHOLE: &str = "main";

    pub(super) fn new(path: PathBuf, arguments: impl IntoIterator<Item = String>) -> TestBinary {
        TestBinary {
            path,
            arguments: arguments.into_iter().collect(),
        }
    }

    pub(super) fn runs(&self) -> Vec<TestRun> {
        self.tests().map_or_else(
            |failure| {
                eprintln!("{failure}");
                vec![TestRun::Unlisted(self.path.clone())]
            },
            |tests| self.listed_runs(tests),
        )
    }

    fn listed_runs(&self, tests: Vec<ListedTest>) -> Vec<TestRun> {
        if tests.is_empty() {
            vec![self.timed(&self.line().arguments(&self.arguments), Self::WHOLE)]
        } else {
            tests
                .into_iter()
                .map(|test| {
                    let line = self
                        .line()
                        .argument(TestArgument::Exact)
                        .argument(&test)
                        .argument(TestArgument::Quiet);
                    self.timed(&line, test)
                })
                .collect()
        }
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

    fn timed(&self, line: &CommandLine, test: impl fmt::Display) -> TestRun {
        let started = Instant::now();
        let ran = line.captured::<Stdout>();
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
