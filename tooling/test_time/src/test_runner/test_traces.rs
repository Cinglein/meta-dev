mod access;

use std::fs::{self, OpenOptions};
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
        let git = own.join(Spawns::GIT_TRACES);
        let starts = own.join(SpawnTrace::STARTS);
        fs::rename(&git, own.join(Spawns::SEALED_GIT_TRACES))
            .and_then(|()| fs::create_dir(&git))
            .and_then(|()| OpenOptions::new().create(true).append(true).open(&starts))
            .and_then(|_| Access::Sealed.applied(&starts))
            .and_then(|()| Access::Sealed.applied(&own))
            .map_err(|error| Failure::Io {
                site: Site::File(own.clone()),
                error,
            })
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
            .try_for_each(|test| Access::Open.applied(&test).map_err(failed))
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;
    use std::fs;
    use std::path::Path;

    use git_trace::Spawns;
    use meta_run::SpawnTrace;
    use task::{CommandLine, Program, Stdout};

    use super::TestTraces;
    use super::access::Access;
    use crate::test_name::TestName;

    const SHELL: &str = "sh";
    const SHOWN: [&str; 2] = ["-c", "printf %s \"$META_SPAWN_TRACE $GIT_TRACE2_EVENT\""];
    const NAME: [&str; 3] = ["p", "deps/it-0a1b", "a"];
    const OWN: &str = "p it a";

    #[test]
    fn each_test_gets_fresh_trace_directories_and_a_start_after_its_seal_is_refused_or_reported() {
        let root = tempfile::tempdir().unwrap();
        let [package, binary, test] = NAME;
        let traces = TestTraces::new(root.path().to_path_buf());
        let name = TestName::new(package, Path::new(binary), &test);
        let shell = CommandLine::new(Program::Executable(SHELL.into())).arguments(SHOWN);
        let shown = traces
            .traced(shell.clone(), &name)
            .and_then(|line| line.captured::<Stdout>());
        let own = root.path().join(OWN);
        let both = format!("{0} {0}/{1}", own.display(), Spawns::GIT_TRACES);
        assert!(shown.ok() == Some(both) && traces.traced(shell, &name).is_err());
        let start = || SpawnTrace::new(&own).noted(OsStr::new(SHELL)).is_ok();
        assert!(start() && traces.sealed(&name).is_ok() && !start());
        let outside = root.path().join(Spawns::GIT_TRACES);
        let written = |directory: &Path| fs::write(directory.join(SHELL), SHELL).is_ok();
        let late_git = written(&own.join(Spawns::GIT_TRACES)) && fs::create_dir(&outside).is_ok();
        let reported = late_git && Spawns::traced(root.path()).is_err();
        let kept =
            Access::Sealed.applied(&outside).is_ok() && TestTraces::opened(root.path()).is_ok();
        assert!(reported && kept && written(&own) && !written(&outside));
    }
}
