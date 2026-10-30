mod test_binary;
pub(crate) mod test_traces;

use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use super::test_runs::TestRuns;
use meta_run::SpawnTrace;
use task::{Failure, Site};
use test_binary::TestBinary;
use test_traces::TestTraces;

#[derive(Debug)]
pub struct TestRunner;

impl TestRunner {
    pub const TASK: &str = "test-runner";
    pub const RUNS: &str = "META_TEST_RUNS";
    const BINARY: &str = "the test binary cargo hands the runner";
    const PACKAGE: &str = "CARGO_PKG_NAME";

    pub fn run(mut arguments: impl Iterator<Item = String>) -> Result<(), Failure> {
        let binary = arguments
            .next()
            .map(PathBuf::from)
            .ok_or(Failure::Unset(Self::BINARY))?;
        let runs = env::var_os(Self::RUNS)
            .map(PathBuf::from)
            .ok_or(Failure::Unset(Self::RUNS))?;
        let package = env::var(Self::PACKAGE).map_err(|_| Failure::Unset(Self::PACKAGE))?;
        let traces =
            env::var_os(SpawnTrace::VARIABLE).map(|root| TestTraces::new(PathBuf::from(root)));
        TestRunner::recorded(&TestBinary::new(binary, package, arguments, traces), &runs)
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

    const CANNOT_LIST: &str = "false";
    const LISTS_NOTHING: &str = "true";
    const FIRST: &str = "unlisted\tfalse\npassed\t";
    const LAST: &str = "\tmain\ttrue\n";

    #[test]
    fn a_binary_that_cannot_list_fails_and_one_that_lists_nothing_runs_once_as_one_test() {
        let runs = tempfile::NamedTempFile::new().unwrap();
        let recorded = |binary: &str| {
            TestRunner::recorded(
                &TestBinary::new(PathBuf::from(binary), binary.to_owned(), [], None),
                runs.path(),
            )
        };
        assert!(recorded(CANNOT_LIST).is_err() && recorded(LISTS_NOTHING).is_ok());
        let read = fs::read_to_string(runs.path()).unwrap();
        assert!(read.starts_with(FIRST) && read.ends_with(LAST), "{read}");
    }
}
