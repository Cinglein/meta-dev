use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use git_trace::{GitProcesses, Spawns};
use meta_run::SpawnTrace;
use task::{CommandLine, Failure, Site};

use crate::test_name::TestName;

#[derive(Debug)]
pub(crate) struct TestTraces(PathBuf);

impl TestTraces {
    pub(crate) fn new(root: PathBuf) -> TestTraces {
        TestTraces(root)
    }

    pub(crate) fn traced(
        &self,
        line: CommandLine,
        binary: &Path,
        test: &impl fmt::Display,
    ) -> Result<CommandLine, Failure> {
        let own = TestName::new(binary, test).within(&self.0);
        let git = own.join(Spawns::GIT_TRACES);
        let made = fs::create_dir(&own).and_then(|()| fs::create_dir(&git));
        made.map_err(|error| Failure::Io {
            site: Site::File(own.clone()),
            error,
        })?;
        Ok(line
            .environment(SpawnTrace::VARIABLE, own)
            .environment(GitProcesses::TARGET, git))
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use task::{CommandLine, Program, Stdout};

    use super::TestTraces;

    const SHELL: &str = "sh";
    const SHOWN: [&str; 2] = ["-c", "printf %s \"$META_SPAWN_TRACE $GIT_TRACE2_EVENT\""];
    const BINARY: &str = "deps/it-0a1b";
    const OWN: &str = "it a";
    const TEST: &str = "a";
    const TRACES: &str = "git-traces";

    #[test]
    fn each_test_gets_fresh_trace_directories_of_its_own_and_a_reused_name_fails() {
        let root = tempfile::tempdir().unwrap();
        let traces = TestTraces::new(root.path().to_path_buf());
        let shell = CommandLine::new(Program::Executable(SHELL.into())).arguments(SHOWN);
        let shown = traces
            .traced(shell.clone(), Path::new(BINARY), &TEST)
            .and_then(|line| line.captured::<Stdout>());
        let own = root.path().join(OWN);
        assert_eq!(
            shown.ok(),
            Some(
                [own.clone(), own.join(TRACES)]
                    .map(|path| path.display().to_string())
                    .join(" ")
            )
        );
        assert!(traces.traced(shell, Path::new(BINARY), &TEST).is_err());
    }
}
