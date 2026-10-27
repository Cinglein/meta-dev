mod built_binary;
mod listed_test;
mod test_argument;

use std::path::PathBuf;
use std::time::Instant;

use built_binary::BuiltBinary;
use listed_test::ListedTest;
use test_argument::TestArgument;

use super::TestTime;
use super::test_timing::TestTiming;
use task::Failure;
use task::Policy;
use task::Selection;
use task::Workspace;
use task::{CommandLine, Program, Stderr, Stdout};

pub(super) struct TestBinary {
    path: PathBuf,
}

impl TestBinary {
    pub(super) fn discover(
        workspace: &Workspace,
        selection: &Selection,
        policy: &Policy,
    ) -> Result<Vec<TestBinary>, Failure> {
        let built = TestTime::cargo_test(workspace.line(Program::Cargo), selection, policy)
            .captured::<Stderr>()?;
        Ok(built
            .lines()
            .filter_map(|line| line.parse::<BuiltBinary>().ok())
            .map(|binary| TestBinary {
                path: workspace.root().join(binary.path()),
            })
            .collect())
    }

    pub(super) fn timings(&self) -> Vec<Result<TestTiming, Failure>> {
        self.tests().map_or_else(
            |failure| vec![Err(failure)],
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

    fn timed(&self, test: ListedTest) -> Result<TestTiming, Failure> {
        let started = Instant::now();
        self.line()
            .argument(TestArgument::Exact)
            .argument(&test)
            .argument(TestArgument::Quiet)
            .captured::<Stdout>()?;
        Ok(TestTiming::new(&self.path, test, started.elapsed()))
    }

    fn line(&self) -> CommandLine {
        CommandLine::new(Program::Executable(self.path.clone()))
    }
}
