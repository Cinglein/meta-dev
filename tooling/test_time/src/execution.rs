use std::fs;

use git_trace::{GitProcesses, Spawns};
use meta_run::SpawnTrace;
use task::{CommandLine, Failure, Site};
use tempfile::TempDir;

use super::test_runner::test_traces::TestTraces;
use super::test_timing::TestTiming;

#[derive(Debug)]
pub(super) enum Execution {
    Timed,
    Counted(TempDir),
}

impl Execution {
    pub(super) fn counting() -> Result<Execution, Failure> {
        let failed = |error| Failure::Io {
            site: Site::Workspace,
            error,
        };
        let root = tempfile::tempdir().map_err(failed)?;
        fs::create_dir(root.path().join(Spawns::GIT_TRACES)).map_err(failed)?;
        Ok(Execution::Counted(root))
    }

    pub(super) fn traced(&self, line: CommandLine) -> CommandLine {
        match self {
            Execution::Timed => line,
            Execution::Counted(traces) => GitProcesses::LEAN.into_iter().fold(
                line.environment(SpawnTrace::VARIABLE, traces.path())
                    .environment(GitProcesses::TARGET, traces.path().join(Spawns::GIT_TRACES)),
                |line, (name, value)| line.environment(name, value),
            ),
        }
    }

    pub(super) fn doctested<Selected>(&self, selected: Option<Selected>) -> Option<Selected> {
        match self {
            Execution::Timed => selected,
            Execution::Counted(_) => None,
        }
    }

    pub(super) fn processes(&self, timings: &[TestTiming]) -> Result<Spawns, Failure> {
        match self {
            Execution::Timed => Ok(Spawns::default()),
            Execution::Counted(traces) => TestTraces::opened(traces.path())
                .and_then(|()| Spawns::traced(traces.path()))
                .and_then(|spawns| spawns.covering(timings.iter().map(TestTiming::name))),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::Duration;

    use git_trace::Spawns;
    use task::{Amount, CommandLine, Program};

    use super::Execution;
    use crate::test_name::TestName;
    use crate::test_runner::test_traces::TestTraces;
    use crate::test_timing::TestTiming;

    const TRACE: &str = "{\"event\":\"version\",\"sid\":\"20261029T0-H1-P1\"}\n";
    const SELECTED: Option<u8> = Some(1);
    const NAME: [&str; 3] = ["p", "deps/it-0a1b", "t"];
    const TWO: usize = 2;
    const ZERO: usize = 0;

    #[test]
    fn a_counted_run_skips_doctests_and_a_timed_run_keeps_them() {
        let counted = Execution::counting().unwrap();
        assert!(
            counted.doctested(SELECTED).is_none()
                && Execution::Timed.doctested(SELECTED) == SELECTED
        );
    }

    #[test]
    fn a_counted_run_counts_git_traces_inside_and_outside_its_tests() {
        let counted = Execution::counting().unwrap();
        let Execution::Counted(root) = &counted else {
            return;
        };
        let [package, binary, test] = NAME;
        let name = TestName::new(package, Path::new(binary), &test);
        let traces = TestTraces::new(root.path().to_path_buf());
        let line = traces.traced(CommandLine::new(Program::Cargo), &name);
        let dirs = [name.within(root.path()), root.path().to_path_buf()];
        let write = |dir: &PathBuf| fs::write(dir.join(Spawns::GIT_TRACES).join(test), TRACE);
        assert!(
            line.is_ok() && dirs.iter().try_for_each(write).is_ok() && traces.sealed(&name).is_ok()
        );
        let timed = [TestTiming::new(name, Duration::ZERO, Duration::ZERO)];
        let git = counted
            .processes(&timed)
            .map(|spawns| spawns.git().amount());
        assert!(git.is_ok_and(|amount| amount == Amount::Count(TWO.into())));
    }

    #[test]
    fn a_counted_run_needs_every_timed_test_traced_and_a_timed_run_counts_nothing() {
        let [package, binary, test] = NAME;
        let name = TestName::new(package, Path::new(binary), &test);
        let timed = [TestTiming::new(name, Duration::ZERO, Duration::ZERO)];
        let counted = Execution::counting().unwrap();
        let git = |execution: &Execution, timed: &[TestTiming]| {
            execution
                .processes(timed)
                .map(|spawns| spawns.git().amount())
        };
        assert!(git(&counted, &timed).is_err() && git(&counted, &[]).is_ok());
        assert!(
            git(&Execution::Timed, &timed).is_ok_and(|amount| amount == Amount::Count(ZERO.into()))
        );
    }
}
