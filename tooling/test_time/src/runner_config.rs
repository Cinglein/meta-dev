use std::env;
use std::fmt;
use std::path::{Path, PathBuf};

use super::test_runner::TestRunner;
use task::{Failure, Site};

#[derive(Debug)]
pub(super) struct RunnerConfig {
    executable: PathBuf,
    target: PathBuf,
}

impl RunnerConfig {
    const KEY: &str = "target.'cfg(all())'.runner";

    pub(super) fn new(executable: PathBuf, target: &Path) -> RunnerConfig {
        RunnerConfig {
            executable,
            target: target.to_path_buf(),
        }
    }

    pub(super) fn current(target: &Path) -> Result<RunnerConfig, Failure> {
        env::current_exe()
            .map(|executable| RunnerConfig::new(executable, target))
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
            "{} = [{}, {}, {}]",
            Self::KEY,
            Self::quoted(&self.executable.to_string_lossy()),
            Self::quoted(TestRunner::TASK),
            Self::quoted(&self.target.to_string_lossy())
        )
    }
}
