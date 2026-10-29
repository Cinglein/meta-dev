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
    use std::collections::BTreeSet;
    use std::path::{Path, PathBuf};
    use std::{env, fs, process};

    use super::{FailureKind, FailureSet};

    const FILE: &str = "a.rs";
    const JOB: &str = "job";
    const SCRATCH: &str = "failure-set-";
    const MISSING: &str = "no failures file";
    const MALFORMED: [&str; 4] = ["", "[]", "{}", r#"{"failures":[{"kind":"flaky"}]}"#];

    impl FailureSet {
        fn linted() -> FailureSet {
            FailureSet::lint(BTreeSet::from([PathBuf::from(FILE)]))
        }
    }

    #[test]
    fn lint_and_golden_sets_name_their_paths_and_no_paths_is_no_failure() {
        let kinds = |set: FailureSet| set.kinds().to_vec();
        let paths = vec![PathBuf::from(FILE)];
        assert_eq!(
            kinds(FailureSet::linted()),
            [FailureKind::Lint {
                paths: paths.clone()
            }]
        );
        assert_eq!(
            kinds(FailureSet::golden(PathBuf::from(FILE))),
            [FailureKind::Golden { paths }]
        );
        assert!(FailureSet::lint(BTreeSet::new()).is_empty());
    }

    #[test]
    fn sets_join_and_only_an_empty_set_falls_back_to_infra() {
        let joined = FailureSet::default()
            .joined(&FailureSet::linted())
            .joined(&FailureSet::linted());
        let infra = |set: FailureSet| set.or_infra(JOB.into()).kinds().len();
        assert_eq!(joined.kinds().len(), 2);
        assert_eq!(
            [infra(FailureSet::default()), infra(FailureSet::linted())],
            [1, 1]
        );
    }

    #[test]
    fn a_set_reads_back_as_written_and_a_missing_file_a_directory_and_malformed_text_are_unattributed()
     {
        let file = env::temp_dir().join(format!("{SCRATCH}{}", process::id()));
        FailureSet::linted().write(&file).unwrap();
        let back = FailureSet::read(&file).ok();
        fs::remove_file(&file).unwrap();
        let told = |path: &Path| {
            FailureSet::read(path)
                .err()
                .map(|failure| failure.to_string().contains(MISSING))
        };
        assert_eq!(back, Some(FailureSet::linted()));
        assert_eq!(
            [told(&file), told(&env::temp_dir())],
            [Some(true), Some(false)]
        );
        assert!(
            MALFORMED
                .iter()
                .all(|json| json.parse::<FailureSet>().is_err())
        );
    }
}
