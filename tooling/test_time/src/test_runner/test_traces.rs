mod access;

use std::fs;
use std::path::{Path, PathBuf};

use git_trace::{GitProcesses, Spawns};
use meta_run::SpawnTrace;
use task::{CommandLine, Failure, Site};

use crate::test_name::TestName;
use access::Access;

#[derive(Debug)]
pub(crate) struct TestTraces(PathBuf);

impl TestTraces {
    pub(crate) fn new(root: PathBuf) -> TestTraces {
        TestTraces(root)
    }

    pub(crate) fn traced(
        &self,
        line: CommandLine,
        name: &TestName,
    ) -> Result<CommandLine, Failure> {
        let own = name.within(&self.0);
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

    pub(crate) fn sealed(&self, name: &TestName) -> Result<(), Failure> {
        let own = name.within(&self.0);
        Self::marked(&own, Access::Sealed)
    }

    pub(crate) fn opened(root: &Path) -> Result<(), Failure> {
        let failed = |error| Failure::Io {
            site: Site::File(root.to_path_buf()),
            error,
        };
        let tests = fs::read_dir(root)
            .map_err(failed)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(failed)?;
        let own = tests
            .iter()
            .map(fs::DirEntry::path)
            .filter(|test| test.is_dir() && !test.ends_with(Spawns::GIT_TRACES));
        own.into_iter()
            .try_for_each(|test| Self::marked(&test, Access::Open))
    }

    fn marked(own: &Path, access: Access) -> Result<(), Failure> {
        let failed = |error| Failure::Io {
            site: Site::File(own.to_path_buf()),
            error,
        };
        let mark = |dir: &Path| access.applied(dir).map_err(failed);
        mark(own).and_then(|()| mark(&own.join(Spawns::GIT_TRACES)))
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;
    use std::path::Path;

    use meta_run::SpawnTrace;
    use task::{CommandLine, Program, Stdout};

    use super::TestTraces;
    use crate::test_name::TestName;

    const SHELL: &str = "sh";
    const SHOWN: [&str; 2] = ["-c", "printf %s \"$META_SPAWN_TRACE $GIT_TRACE2_EVENT\""];
    const BINARY: &str = "deps/it-0a1b";
    const OWN: &str = "p it a";
    const TEST: &str = "a";
    const TRACES: &str = "git-traces";

    #[test]
    fn each_test_gets_fresh_trace_directories_sealed_after_it_and_a_reused_name_fails() {
        let root = tempfile::tempdir().unwrap();
        let (traces, name) = (
            TestTraces::new(root.path().to_path_buf()),
            TestName::new("p", Path::new(BINARY), &TEST),
        );
        let shell = CommandLine::new(Program::Executable(SHELL.into())).arguments(SHOWN);
        let shown = traces
            .traced(shell.clone(), &name)
            .and_then(|line| line.captured::<Stdout>());
        let own = root.path().join(OWN);
        let shown_both = [own.clone(), own.join(TRACES)]
            .map(|path| path.display().to_string())
            .join(" ");
        assert!(shown.ok() == Some(shown_both) && traces.traced(shell, &name).is_err());
        let late = || SpawnTrace::new(&own).noted(OsStr::new(SHELL)).is_ok();
        assert!(traces.sealed(&name).is_ok() && !late());
        assert!(TestTraces::opened(root.path()).is_ok() && late());
    }
}
