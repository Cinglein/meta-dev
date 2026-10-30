use std::ffi::OsStr;
use std::fmt;
use std::path::{Path, PathBuf};

use task::{CrateName, FailureKind};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TestName {
    package: String,
    binary: PathBuf,
    test: String,
}

impl TestName {
    pub(crate) fn new(package: &str, binary: &Path, test: &impl fmt::Display) -> TestName {
        TestName {
            package: package.to_owned(),
            binary: binary.to_path_buf(),
            test: test.to_string(),
        }
    }

    pub(crate) fn within(&self, root: &Path) -> PathBuf {
        root.join(self.to_string())
    }

    pub(crate) fn shown(&self) -> String {
        format!("{} {}", self.stem(), self.test)
    }

    pub(crate) fn fields(&self, separator: char) -> String {
        format!(
            "{}{separator}{}{separator}{}",
            self.package,
            self.test,
            self.binary.display()
        )
    }

    pub(crate) fn failure(&self) -> FailureKind {
        FailureKind::Test {
            crate_name: CrateName::from(self.binary.as_path()),
            name: self.test.as_str().into(),
        }
    }

    fn stem(&self) -> &str {
        let name = self.binary.file_name().and_then(OsStr::to_str);
        name.and_then(|name| name.rsplit_once('-'))
            .map(|(stem, _)| stem)
            .unwrap_or_default()
    }
}

impl fmt::Display for TestName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {} {}", self.package, self.stem(), self.test)
    }
}
