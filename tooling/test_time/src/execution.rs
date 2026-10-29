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
