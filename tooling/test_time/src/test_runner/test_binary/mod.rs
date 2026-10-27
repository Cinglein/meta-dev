mod listed_test;
mod listing;
mod test_argument;

use std::fmt;
use std::path::PathBuf;
use std::time::Instant;

use listed_test::ListedTest;
use listing::Listing;
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
        self.listing().map_or_else(
            |failure| {
                eprintln!("{failure}");
                vec![TestRun::Unlisted(self.path.clone())]
            },
            |listing| listing.runs(self),
        )
    }

    pub(in crate::test_runner::test_binary) fn listed(&self, test: ListedTest) -> TestRun {
        let line = self
            .line()
            .argument(TestArgument::Exact)
            .argument(&test)
            .argument(TestArgument::Quiet);
        self.timed(&line, test)
    }

    pub(in crate::test_runner::test_binary) fn whole(&self) -> TestRun {
        self.timed(&self.line().arguments(&self.arguments), Self::WHOLE)
    }

    fn listing(&self) -> Result<Listing, Failure> {
        self.line()
            .argument(TestArgument::List)
            .captured::<Stdout>()?
            .parse()
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
