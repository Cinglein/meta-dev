use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use serde::{Deserialize, Serialize};
use task::{BaselineFile, BranchName, BudgetId, Site};

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

    pub fn admitted(root: &Path, file: &BaselineFile, base: &BranchName) -> Result<Baseline, Failure> {
        Self::kept_since(root, file, base, |prior| Some(prior.baseline().unwrap_or(file).clone()))
    }

    pub fn budgeted(
        root: &Path,
        budget: &BudgetId,
        file: &BaselineFile,
        base: &BranchName,
    ) -> Result<Baseline, Failure> {
        Self::kept_since(root, file, base, |prior| prior.budget_baseline(budget).cloned())
    }

    fn kept_since(
        root: &Path,
        file: &BaselineFile,
        base: &BranchName,
        prior_file: impl FnOnce(&PriorPolicy) -> Option<BaselineFile>,
    ) -> Result<Baseline, Failure> {
        let allowed = Self::read(&file.within(root))?;
        let revision = Revision::merge_base(root, base)?;
        match prior_file(&PriorPolicy::read_at(&revision)?) {
            Some(prior) => allowed.kept(revision.present(prior.path())?.as_deref()),
            None => Ok(allowed),
        }
    }

    fn kept(self, prior: Option<&str>) -> Result<Baseline, Failure> {
        let prior: Baseline = prior.map(toml::from_str).transpose()?.unwrap_or_default();
        let raised: Vec<Drift> = self.drifts(&prior).into_iter().filter(Drift::grew).collect();
        if raised.is_empty() { Ok(self) } else { Err(Failure::Raised(Raised::new(raised))) }
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
    use crate::task::prior_policy::PriorPolicy;
    use settings::EmbeddedDefaults;
    use task::{BaselineFile, BudgetId, Policy};

    const SELF_BASELINE: &str = "self/lint-baseline.toml";
    const LINT_FINDINGS: &str = "lint-findings";
    const UNREADABLE: &str = "retired = 1\n";
    const BASELINE: &str = "lint-baseline.toml";
    const MOVED: &str = "self/moved.toml";
    const MOVED_POLICY: &str = "baseline = \"self/moved.toml\"\n";
    const ONE: &str = "crate-graph = 1\n";
    const TWO: &str = "crate-graph = 2\n";
    const THREE: &str = "crate-graph = 1\nno-comments = 2\n";
    const GREW: &str = "crate-graph: 2 found, baseline 1";
    const RAISED: &str = "baseline raised above the merge base:\ncrate-graph: 2 found, baseline 1";

    #[test]
    fn a_raised_baseline_is_refused_against_the_merge_base_even_after_a_move() {
        let two = || toml::from_str::<Baseline>(TWO).unwrap();
        let raised = two().kept(Some(ONE)).map_err(|failure| failure.to_string());
        assert!(raised.err().as_deref() == Some(RAISED) && two().kept(Some(TWO)).is_ok() && two().kept(None).is_err());
        let budgeted = Policy::DEFAULTS.replace(SELF_BASELINE, MOVED);
        let moved = [MOVED_POLICY, &budgeted, &format!("{UNREADABLE}{budgeted}")].map(|policy| {
            let prior: PriorPolicy = toml::from_str(policy).unwrap();
            let budgeted = prior.budget_baseline(&BudgetId::from(LINT_FINDINGS.to_owned()));
            let held = (budgeted == Some(&BaselineFile::from(MOVED))) == (policy != MOVED_POLICY);
            held && prior.baseline().unwrap_or(&BaselineFile::from(BASELINE)) == &BaselineFile::from(MOVED)
        });
        assert!(moved.iter().all(|moved| *moved));
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
        assert_eq!(toml::from_str::<Baseline>(THREE).unwrap().total().to_string(), "3");
    }

    #[test]
    fn a_missing_baseline_reads_empty_a_written_one_reads_back_and_a_directory_fails() {
        let fixture = Fixture::new();
        let written: Baseline = toml::from_str(THREE).unwrap();
        assert_eq!(Baseline::read(&fixture.path(BASELINE)).ok(), Some(Baseline::default()));
        written.write(&fixture.path(BASELINE)).unwrap();
        assert_eq!(Baseline::read(&fixture.path(BASELINE)).ok(), Some(written));
        assert!(matches!(Baseline::read(fixture.root()), Err(Failure::Task(_))));
    }
}
