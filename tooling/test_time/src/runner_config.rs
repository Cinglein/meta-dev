use std::env;
use std::fmt;
use std::path::PathBuf;

use super::test_runner::TestRunner;
use task::{Failure, Site};

#[derive(Debug)]
pub(super) struct RunnerConfig {
    executable: PathBuf,
}

impl RunnerConfig {
    const KEY: &str = "target.'cfg(all())'.runner";

    pub(super) fn new(executable: PathBuf) -> RunnerConfig {
        RunnerConfig { executable }
    }

    pub(super) fn current() -> Result<RunnerConfig, Failure> {
        env::current_exe()
            .map(RunnerConfig::new)
            .map_err(|error| Failure::Io {
                site: Site::Workspace,
                error,
            })
    }

    fn quoted(text: &str) -> String {
        format!("\"{}\"", text.replace('\\', "\\\\").replace('"', "\\\""))
    }
}

impl fmt::Display for RunnerConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} = [{}, {}]",
            Self::KEY,
            Self::quoted(&self.executable.to_string_lossy()),
            Self::quoted(TestRunner::TASK)
        )
    }
}
