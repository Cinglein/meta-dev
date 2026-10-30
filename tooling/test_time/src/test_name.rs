use std::fmt;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub(crate) struct TestName<'test, Test: fmt::Display> {
    binary: &'test Path,
    test: &'test Test,
}

impl<'test, Test: fmt::Display> TestName<'test, Test> {
    pub(crate) fn new(binary: &'test Path, test: &'test Test) -> Self {
        TestName { binary, test }
    }

    pub(crate) fn within(&self, root: &Path) -> PathBuf {
        root.join(self.to_string())
    }
}

impl<Test: fmt::Display> fmt::Display for TestName<'_, Test> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = self.binary.file_name().map(|name| name.to_string_lossy());
        let stem = name
            .as_ref()
            .and_then(|name| name.rsplit_once('-'))
            .map(|(stem, _)| stem);
        write!(formatter, "{} {}", stem.unwrap_or_default(), self.test)
    }
}
