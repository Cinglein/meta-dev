use std::fmt;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct TestTiming {
    binary: PathBuf,
    test: String,
    elapsed: Duration,
}

impl TestTiming {
    pub(super) fn new(binary: &Path, test: impl fmt::Display, elapsed: Duration) -> TestTiming {
        TestTiming {
            binary: binary.to_path_buf(),
            test: test.to_string(),
            elapsed,
        }
    }

    pub(super) const fn elapsed(&self) -> Duration {
        self.elapsed
    }

    pub(super) fn binary(&self) -> &Path {
        &self.binary
    }

    pub(super) fn test(&self) -> &str {
        &self.test
    }
}

impl fmt::Display for TestTiming {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let stem = self
            .binary
            .file_name()
            .map(|name| name.to_string_lossy())
            .and_then(|name| name.rsplit_once('-').map(|(stem, _)| stem.to_owned()))
            .unwrap_or_default();
        write!(formatter, "{stem} {} took {:.1?}", self.test, self.elapsed)
    }
}
