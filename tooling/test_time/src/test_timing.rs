use std::fmt;
use std::path::Path;
use std::time::Duration;

#[derive(Clone, Debug)]
pub(super) struct TestTiming {
    binary: String,
    test: String,
    elapsed: Duration,
}

impl TestTiming {
    pub(super) fn new(binary: &Path, test: impl fmt::Display, elapsed: Duration) -> TestTiming {
        TestTiming {
            binary: binary
                .file_name()
                .map(|name| name.to_string_lossy())
                .and_then(|name| name.rsplit_once('-').map(|(stem, _)| stem.to_owned()))
                .unwrap_or_default(),
            test: test.to_string(),
            elapsed,
        }
    }

    pub(super) const fn elapsed(&self) -> Duration {
        self.elapsed
    }
}

impl fmt::Display for TestTiming {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} {} took {:.1?}",
            self.binary, self.test, self.elapsed
        )
    }
}
