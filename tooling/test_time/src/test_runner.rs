mod test_binary;

use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use super::test_runs::TestRuns;
use task::{CommandLine, Failure, Program, Site};
use test_binary::TestBinary;

#[derive(Debug)]
pub struct TestRunner;

impl TestRunner {
    pub const TASK: &str = "test-runner";
    pub const RUNS: &str = "META_TEST_RUNS";
    const BINARY: &str = "the test binary cargo hands the runner";
    const TARGET: &str = "the target directory in the runner config";

    pub fn run(mut arguments: impl Iterator<Item = String>) -> Result<(), Failure> {
        let target = arguments
            .next()
            .map(PathBuf::from)
            .ok_or(Failure::Unset(Self::TARGET))?;
        let binary = arguments
            .next()
            .map(PathBuf::from)
            .ok_or(Failure::Unset(Self::BINARY))?;
        if binary.starts_with(&target) {
            let runs = env::var_os(Self::RUNS)
                .map(PathBuf::from)
                .ok_or(Failure::Unset(Self::RUNS))?;
            TestRunner::recorded(&TestBinary::new(binary), &runs)
        } else {
            CommandLine::new(Program::Executable(binary))
                .arguments(arguments)
                .run()
        }
    }

    fn recorded(binary: &TestBinary, runs: &Path) -> Result<(), Failure> {
        let ran = binary.runs();
        let io = |error| Failure::Io {
            site: Site::File(runs.to_path_buf()),
            error,
        };
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(runs)
            .map_err(io)?;
        ran.iter()
            .try_for_each(|run| writeln!(&file, "{run}"))
            .map_err(io)?;
        ran.into_iter().collect::<TestRuns>().passed().map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use super::{TestBinary, TestRunner};
    use crate::test_run::TestRun;

    const CANNOT_LIST: &str = "false";
    const LISTS_NOTHING: &str = "true";

    #[test]
    fn a_binary_that_cannot_list_is_recorded_as_unlisted_and_fails_and_an_empty_one_passes() {
        let runs = tempfile::NamedTempFile::new().unwrap();
        let recorded = |binary: &str| {
            TestRunner::recorded(&TestBinary::new(PathBuf::from(binary)), runs.path())
        };
        assert!(recorded(CANNOT_LIST).is_err() && recorded(LISTS_NOTHING).is_ok());
        let read = fs::read_to_string(runs.path()).unwrap();
        let runs: Vec<TestRun> = read.lines().filter_map(|line| line.parse().ok()).collect();
        assert_eq!(runs, [TestRun::Unlisted(PathBuf::from(CANNOT_LIST))]);
    }
}
