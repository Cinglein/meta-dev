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
pub use unattributed::Unattributed;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FailureSet {
    failures: Vec<FailureKind>,
}

impl FailureSet {
    const UNFINISHED: &str = "unfinished\n";
    const STAGING: &str = "staging";

    #[must_use]
    pub fn recording(mut self, kind: FailureKind) -> FailureSet {
        self.failures.push(kind);
        self
    }

    #[must_use]
    pub fn lint(paths: BTreeSet<PathBuf>) -> FailureSet {
        FailureSet {
            failures: Some(paths)
                .filter(|paths| !paths.is_empty())
                .map(|paths| FailureKind::Lint {
                    paths: paths.into_iter().collect(),
                })
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

    pub fn begin(path: &Path) -> io::Result<()> {
        Self::replace(path, Self::UNFINISHED)
    }

    pub fn write(&self, path: &Path) -> io::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        Self::replace(path, &[json.as_str(), "\n"].concat())
    }

    fn replace(path: &Path, text: &str) -> io::Result<()> {
        let staged = path.with_extension(Self::STAGING);
        fs::write(&staged, text)?;
        fs::rename(&staged, path)
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
    use std::collections::BTreeSet;
    use std::path::{Path, PathBuf};
    use std::{env, fs, process};

    use super::{FailureKind, FailureSet};

    const FILE: &str = "a.rs";
    const JOB: &str = "job";
    const SCRATCH: &str = "failure-set-";
    const MISSING: &str = "no failures file";
    const SHAPE: &str = "not the shape expected";
    const GONE: &str = "/nonexistent/failures.json";
    const MALFORMED: [&str; 4] = ["", "[]", "{}", r#"{"failures":[{"kind":"flaky"}]}"#];

    impl FailureSet {
        fn linted() -> FailureSet {
            FailureSet::lint(BTreeSet::from([PathBuf::from(FILE)]))
        }
    }

    #[test]
    fn sets_name_their_paths_join_and_only_an_empty_set_falls_back_to_infra() {
        let paths = vec![PathBuf::from(FILE)];
        let infra = |set: FailureSet| set.or_infra(JOB.into()).kinds().len();
        let joined = FailureSet::linted().joined(&FailureSet::linted());
        assert_eq!(FailureSet::linted().kinds(), [FailureKind::Lint { paths }]);
        assert!(FailureSet::lint(BTreeSet::new()).is_empty());
        assert_eq!(
            [
                joined.kinds().len(),
                infra(FailureSet::default()),
                infra(joined)
            ],
            [2, 1, 2]
        );
    }

    #[test]
    fn a_begun_file_is_unattributed_until_a_set_replaces_it() {
        let file = env::temp_dir().join(format!("{SCRATCH}{}", process::id()));
        FailureSet::begin(&file).unwrap();
        let begun = FailureSet::read(&file)
            .err()
            .map(|failure| failure.to_string().contains(SHAPE));
        FailureSet::linted().write(&file).unwrap();
        let back = FailureSet::read(&file).ok();
        fs::remove_file(&file).unwrap();
        assert_eq!(begun, Some(true));
        assert_eq!(back, Some(FailureSet::linted()));
    }

    #[test]
    fn a_missing_file_a_directory_and_malformed_text_are_all_unattributed() {
        let told = |path: &Path| {
            FailureSet::read(path)
                .err()
                .map(|failure| failure.to_string().contains(MISSING))
        };
        let malformed = MALFORMED
            .iter()
            .all(|json| json.parse::<FailureSet>().is_err());
        assert_eq!(
            [told(Path::new(GONE)), told(&env::temp_dir())],
            [Some(true), Some(false)]
        );
        assert!(malformed);
    }
}
