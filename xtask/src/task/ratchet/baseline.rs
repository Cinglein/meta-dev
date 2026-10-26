use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use serde::{Deserialize, Serialize};
use task::{BaselineFile, Site};

use crate::task::failure::Failure;
use crate::task::prior_policy::PriorPolicy;
use crate::task::ratchet::drift::Drift;
use crate::task::ratchet::raised::Raised;
use crate::task::ratchet::violation_count::ViolationCount;
use crate::task::revision::Revision;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Baseline(BTreeMap<String, ViolationCount>);

impl Baseline {
    pub fn read(path: &Path) -> Result<Baseline, Failure> {
        match fs::read_to_string(path) {
            Ok(text) => Ok(toml::from_str(&text)?),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(Baseline::default()),
            Err(error) => Err(Failure::from(task::Failure::Io { site: Site::File(path.to_path_buf()), error })),
        }
    }

    pub fn admitted(root: &Path, file: &BaselineFile) -> Result<Baseline, Failure> {
        let allowed = Self::read(&file.within(root))?;
        let revision = Revision::merge_base(root)?;
        let prior_policy = PriorPolicy::read_at(&revision);
        let prior_file = prior_policy.baseline().unwrap_or(file);
        let prior: Baseline =
            revision.file(prior_file.path()).map(|text| toml::from_str(&text)).transpose()?.unwrap_or_default();
        let raised: Vec<Drift> = allowed.drifts(&prior).into_iter().filter(Drift::grew).collect();
        if raised.is_empty() { Ok(allowed) } else { Err(Failure::Raised(Raised::new(raised))) }
    }

    pub fn write(&self, path: &Path) -> Result<(), Failure> {
        let text = toml::to_string(self)?;
        fs::write(path, text)
            .map_err(|error| Failure::from(task::Failure::Io { site: Site::File(path.to_path_buf()), error }))
    }

    pub fn drifts(&self, allowed: &Baseline) -> Vec<Drift> {
        let lints: BTreeSet<&String> = self.0.keys().chain(allowed.0.keys()).collect();
        let count = |baseline: &Baseline, lint: &String| baseline.0.get(lint).copied().unwrap_or_default();
        lints.into_iter().filter_map(|lint| Drift::between(lint, count(self, lint), count(allowed, lint))).collect()
    }

    pub fn total(&self) -> ViolationCount {
        self.0.values().sum()
    }
}

impl<'lint> FromIterator<(&'lint str, ViolationCount)> for Baseline {
    fn from_iter<Counts: IntoIterator<Item = (&'lint str, ViolationCount)>>(counts: Counts) -> Baseline {
        Baseline(
            counts
                .into_iter()
                .filter(|(_, count)| *count != ViolationCount::default())
                .map(|(lint, count)| (lint.to_owned(), count))
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::Baseline;
    use crate::task::failure::Failure;
    use crate::task::fixture::Fixture;
    use task::BaselineFile;

    const BASELINE: &str = "lint-baseline.toml";
    const ONE: &str = "crate-graph = 1\n";
    const TWO: &str = "crate-graph = 2\n";
    const GREW: &str = "crate-graph: 2 found, baseline 1";

    #[test]
    fn a_raised_baseline_is_refused_against_the_merge_base() {
        let fixture = Fixture::new();
        fixture.write(BASELINE, ONE).branch();
        fixture.write(BASELINE, TWO);
        let admitted = Baseline::admitted(fixture.root(), &BaselineFile::from(BASELINE));
        assert!(matches!(admitted, Err(Failure::Raised(_))));
    }

    #[test]
    fn growth_names_the_lint_and_both_counts() {
        let empty = Baseline::default();
        let grown: Baseline = toml::from_str(TWO).unwrap();
        let allowed: Baseline = toml::from_str(ONE).unwrap();
        let drifts: Vec<String> = grown.drifts(&allowed).iter().map(ToString::to_string).collect();
        assert_eq!(drifts, [GREW]);
        assert!(
            grown.drifts(&allowed).iter().all(super::Drift::grew)
                && empty.drifts(&allowed).iter().all(|drift| !drift.grew())
        );
    }
}
