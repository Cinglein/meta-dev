use std::fmt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use super::clock::Clock;
use task::CrateName;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct TestTiming {
    binary: PathBuf,
    test: String,
    elapsed: Duration,
    cpu: Duration,
}

impl TestTiming {
    pub(super) fn new(
        binary: &Path,
        test: impl fmt::Display,
        elapsed: Duration,
        cpu: Duration,
    ) -> TestTiming {
        TestTiming {
            binary: binary.to_path_buf(),
            test: test.to_string(),
            elapsed,
            cpu,
        }
    }

    pub(super) const fn spent(&self, clock: Clock) -> Duration {
        match clock {
            Clock::Wall => self.elapsed,
            Clock::Cpu => self.cpu,
        }
    }

    pub(super) fn crate_name(&self) -> CrateName {
        let stem = self
            .binary
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
            .unwrap_or_default();
        let Ok(name) = stem
            .rsplit_once('-')
            .map_or(stem.as_str(), |(name, _)| name)
            .parse();
        name
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
        write!(
            formatter,
            "{stem} {} took {:.1?}, cpu {:.1?}",
            self.test, self.elapsed, self.cpu
        )
    }
}
