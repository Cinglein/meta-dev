use git_trace::GitProcesses;
use task::{CommandLine, Failure, Site};
use tempfile::TempDir;

#[derive(Debug)]
pub(super) enum Execution {
    Timed,
    Counted(TempDir),
}

impl Execution {
    pub(super) fn counting() -> Result<Execution, Failure> {
        tempfile::tempdir()
            .map(Execution::Counted)
            .map_err(|error| Failure::Io {
                site: Site::Workspace,
                error,
            })
    }

    pub(super) fn traced(&self, line: CommandLine) -> CommandLine {
        match self {
            Execution::Timed => line,
            Execution::Counted(traces) => GitProcesses::LEAN.into_iter().fold(
                line.environment(GitProcesses::TARGET, traces.path()),
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

    pub(super) fn processes(&self) -> Result<GitProcesses, Failure> {
        match self {
            Execution::Timed => Ok(GitProcesses::default()),
            Execution::Counted(traces) => GitProcesses::traced(traces.path()),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use task::Amount;

    use super::Execution;

    const TRACE: &str = "{\"event\":\"version\",\"sid\":\"20261029T0-H1-P1\"}\n";
    const FILE: &str = "trace";
    const SELECTED: Option<u8> = Some(1);
    const ONE: usize = 1;
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
    fn a_counted_run_counts_what_the_traces_hold_and_a_timed_run_counts_nothing() {
        let counted = Execution::counting().unwrap();
        let Execution::Counted(traces) = &counted else {
            return;
        };
        fs::write(traces.path().join(FILE), TRACE).unwrap();
        let one = counted.processes().map(|processes| processes.amount());
        let none = Execution::Timed
            .processes()
            .map(|processes| processes.amount());
        assert!(
            one.is_ok_and(|amount| amount == Amount::Count(ONE.into()))
                && none.is_ok_and(|amount| amount == Amount::Count(ZERO.into()))
        );
    }
}
