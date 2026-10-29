use std::fs;
use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::failure_kind::FailureKind;
use crate::unattributed::Unattributed;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FailureSet {
    failures: Vec<FailureKind>,
}

impl FailureSet {
    #[must_use]
    pub fn with(mut self, kind: FailureKind) -> FailureSet {
        self.failures.push(kind);
        self
    }

    pub fn extend(&mut self, other: FailureSet) {
        self.failures.extend(other.failures);
    }

    #[must_use]
    pub fn joined(mut self, other: FailureSet) -> FailureSet {
        self.extend(other);
        self
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.failures.is_empty()
    }

    #[must_use]
    pub fn kinds(&self) -> &[FailureKind] {
        &self.failures
    }

    pub fn parse(json: &str) -> Result<FailureSet, Unattributed> {
        serde_json::from_str(json).map_err(Unattributed::Malformed)
    }

    pub fn read(path: &Path) -> Result<FailureSet, Unattributed> {
        match fs::read_to_string(path) {
            Ok(json) => FailureSet::parse(&json),
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

impl std::fmt::Display for FailureSet {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let json = serde_json::to_string_pretty(self).map_err(|_| std::fmt::Error)?;
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
        let set = FailureSet::default().with(FailureKind::Infra { job: "ci".into() });
        let back = [FailureSet::default(), set]
            .map(|set| FailureSet::parse(&set.to_string()).ok() == Some(set));
        let refused = UNREADABLE
            .iter()
            .all(|json| FailureSet::parse(json).is_err());
        assert!(back == [true, true] && refused && FailureSet::read(Path::new(GONE)).is_err());
    }
}
