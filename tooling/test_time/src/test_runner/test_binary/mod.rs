mod listed_test;
mod listing;
mod test_argument;

use std::fmt;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use listed_test::ListedTest;
use listing::Listing;
use test_argument::TestArgument;

use crate::test_name::TestName;
use crate::test_run::TestRun;
use crate::test_runner::test_traces::TestTraces;
use crate::test_timing::TestTiming;
use rusage::ChildrenCpu;
use task::Failure;
use task::{CommandLine, Program, Site, Stdout};

#[derive(Debug)]
pub(super) struct TestBinary {
    path: PathBuf,
    package: String,
    arguments: Vec<String>,
    traces: Option<TestTraces>,
}

impl TestBinary {
    const WHOLE: &str = "main";

    pub(super) fn new(
        path: PathBuf,
        package: String,
        arguments: impl IntoIterator<Item = String>,
        traces: Option<TestTraces>,
    ) -> TestBinary {
        TestBinary {
            path,
            package,
            arguments: arguments.into_iter().collect(),
            traces,
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
        let cpu = || {
            ChildrenCpu::now().map_err(|error| Failure::Io {
                site: Site::Workspace,
                error,
            })
        };
        let name = TestName::new(&self.package, &self.path, &test);
        let traced = self.traces.as_ref().map_or_else(
            || Ok(line.clone()),
            |traces| traces.traced(line.clone(), &name),
        );
        let before = cpu();
        let started = Instant::now();
        let ran = traced.and_then(|line| line.captured::<Stdout>());
        let elapsed = started.elapsed();
        let spent = before.and_then(|before| cpu().map(|after| after.since(before)));
        let sealed = self
            .traces
            .as_ref()
            .map_or(Ok(()), |traces| traces.sealed(&name));
        let timing = |cpu| TestTiming::new(name.clone(), elapsed, cpu);
        ran.and(sealed)
            .and(spent)
            .inspect_err(|failure| eprintln!("{failure}"))
            .map_or_else(
                |_| TestRun::Failed(timing(Duration::ZERO)),
                |cpu| TestRun::Passed(timing(cpu)),
            )
    }

    fn line(&self) -> CommandLine {
        CommandLine::new(Program::Executable(self.path.clone()))
    }
}
