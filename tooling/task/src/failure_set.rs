mod unattributed;

use std::collections::BTreeSet;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::check_name::CheckName;
use crate::failure_kind::FailureKind;
use unattributed::Unattributed;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FailureSet {
    failures: Vec<FailureKind>,
}

impl FailureSet {
    #[must_use]
    pub fn recording(mut self, kind: FailureKind) -> FailureSet {
        self.failures.push(kind);
        self
    }

    #[must_use]
    pub fn lint(paths: BTreeSet<PathBuf>) -> FailureSet {
        Self::over(paths, |paths| FailureKind::Lint { paths })
    }

    #[must_use]
    pub fn golden(path: PathBuf) -> FailureSet {
        Self::over(BTreeSet::from([path]), |paths| FailureKind::Golden {
            paths,
        })
    }

    fn over(paths: BTreeSet<PathBuf>, kind: fn(Vec<PathBuf>) -> FailureKind) -> FailureSet {
        FailureSet {
            failures: Some(paths)
                .filter(|paths| !paths.is_empty())
                .map(|paths| kind(paths.into_iter().collect()))
                .into_iter()
                .collect(),
        }
    }

    pub fn extend(&mut self, other: FailureSet) {
        self.failures.extend(other.failures);
    }

    #[must_use]
    pub fn joined(mut self, other: &FailureSet) -> FailureSet {
        self.extend(other.clone());
        self
    }

    #[must_use]
    pub fn or_infra(self, job: CheckName) -> FailureSet {
        if self.failures.is_empty() {
            self.recording(FailureKind::Infra { job })
        } else {
            self
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.failures.is_empty()
    }

    #[must_use]
    pub fn kinds(&self) -> &[FailureKind] {
        &self.failures
    }

    pub fn read(path: &Path) -> Result<FailureSet, Unattributed> {
        match fs::read_to_string(path) {
            Ok(json) => json.parse(),
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                Err(Unattributed::Missing(path.to_path_buf()))
            }
            Err(error) => Err(Unattributed::Unreadable {
                path: path.to_path_buf(),
                error,
            }),
        }
    }

    pub fn write(&self, path: &Path) -> io::Result<()> {
        fs::write(path, self.to_string())
    }
}

impl FromStr for FailureSet {
    type Err = Unattributed;

    fn from_str(json: &str) -> Result<FailureSet, Unattributed> {
        serde_json::from_str(json).map_err(Unattributed::Malformed)
    }
}

impl fmt::Display for FailureSet {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let json = serde_json::to_string_pretty(self).map_err(|_| fmt::Error)?;
        writeln!(formatter, "{json}")
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{FailureKind, FailureSet};

    const GONE: &str = "/nonexistent/failures.json";
    const UNREADABLE: [&str; 4] = ["", "[]", "{}", r#"{"failures":[{"kind":"flaky"}]}"#];

    #[test]
    fn a_set_reads_back_as_written_and_a_missing_or_malformed_file_is_unattributed() {
        let set = FailureSet::default().recording(FailureKind::Infra { job: "ci".into() });
        let back =
            [FailureSet::default(), set].map(|set| set.to_string().parse().ok() == Some(set));
        let refused = UNREADABLE
            .iter()
            .all(|json| json.parse::<FailureSet>().is_err());
        assert!(back == [true, true] && refused && FailureSet::read(Path::new(GONE)).is_err());
    }
}
