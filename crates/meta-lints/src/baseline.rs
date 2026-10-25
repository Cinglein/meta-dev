use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::count::Count;
use crate::drift::Drift;
use crate::failure::Failure;
use crate::site::Site;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Baseline(BTreeMap<String, BTreeMap<String, Count>>);

impl Baseline {
    pub fn read(path: &Path) -> Result<Option<Baseline>, Failure> {
        match fs::read_to_string(path) {
            Ok(text) => Self::parse(&text, Site::of_path(path)).map(Some),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
            Err(error) => Err(Failure::Io { site: Site::of_path(path), error }),
        }
    }

    pub fn parse(text: &str, site: Site) -> Result<Baseline, Failure> {
        toml::from_str(text).map_err(|error| Failure::Baseline { site, error: Box::new(error) })
    }

    pub fn write(&self, path: &Path) -> Result<(), Failure> {
        let text = toml::to_string(self).map_err(Failure::Serialize)?;
        fs::write(path, text).map_err(|error| Failure::Io { site: Site::of_path(path), error })
    }

    pub fn counted<'name>(files: impl Iterator<Item = &'name str>) -> BTreeMap<String, Count> {
        files.fold(BTreeMap::new(), |mut counts, file| {
            counts.entry(file.to_owned()).and_modify(|count| *count = *count + Count::ONE).or_insert(Count::ONE);
            counts
        })
    }

    pub fn drifts(&self, allowed: &Baseline) -> Vec<Drift> {
        let found = self.totals();
        let budget = allowed.totals();
        let rules: BTreeSet<&str> = found.keys().chain(budget.keys()).copied().collect();
        let total = |totals: &BTreeMap<&str, Count>, rule: &str| totals.get(rule).copied().unwrap_or_default();
        rules
            .into_iter()
            .map(|rule| Drift::new(rule, total(&found, rule), total(&budget, rule)))
            .filter(Drift::moved)
            .collect()
    }

    pub fn grown_files(&self, allowed: &Baseline, rule: &str) -> Vec<&str> {
        let budget = allowed.0.get(rule);
        self.0
            .get(rule)
            .into_iter()
            .flatten()
            .filter(|(file, count)| budget.and_then(|files| files.get(*file)).is_none_or(|allowed| *count > allowed))
            .map(|(file, _)| file.as_str())
            .collect()
    }

    pub fn totals(&self) -> BTreeMap<&str, Count> {
        self.0.iter().map(|(rule, files)| (rule.as_str(), files.values().sum())).collect()
    }
}

impl FromIterator<(String, BTreeMap<String, Count>)> for Baseline {
    fn from_iter<Entries: IntoIterator<Item = (String, BTreeMap<String, Count>)>>(entries: Entries) -> Baseline {
        Baseline(entries.into_iter().filter(|(_, files)| !files.is_empty()).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::Baseline;

    const ALLOWED: &str = "[no-comments]\n\"a.rs\" = 2\n";
    const MOVED: &str = "[no-comments]\n\"b.rs\" = 2\n";
    const GROWN: &str = "[no-comments]\n\"a.rs\" = 2\n\"b.rs\" = 1\n";
    const RULE: &str = "no-comments";
    const DRIFT: &str = "no-comments: 3 found, baseline 2";
    const GROWN_FILE: &str = "b.rs";

    #[test]
    fn a_moved_file_keeps_the_totals() {
        let allowed: Baseline = toml::from_str(ALLOWED).expect("valid baseline");
        let moved: Baseline = toml::from_str(MOVED).expect("valid baseline");
        assert!(moved.drifts(&allowed).is_empty());
    }

    #[test]
    fn growth_names_the_rule_and_the_grown_file() {
        let allowed: Baseline = toml::from_str(ALLOWED).expect("valid baseline");
        let grown: Baseline = toml::from_str(GROWN).expect("valid baseline");
        let drifts: Vec<String> = grown.drifts(&allowed).iter().map(ToString::to_string).collect();
        assert_eq!(drifts, [DRIFT]);
        assert_eq!(grown.grown_files(&allowed, RULE), [GROWN_FILE]);
    }
}
