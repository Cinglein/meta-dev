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
        let site = || Site::of_path(path);
        match fs::read_to_string(path) {
            Ok(text) => toml::from_str(&text)
                .map(Some)
                .map_err(|error| Failure::Baseline { site: site(), error: Box::new(error) }),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
            Err(error) => Err(Failure::Io { site: site(), error }),
        }
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
        let rules: BTreeSet<&String> = self.0.keys().chain(allowed.0.keys()).collect();
        rules
            .into_iter()
            .flat_map(|rule| Self::rule_drifts(rule, self.0.get(rule), allowed.0.get(rule)))
            .filter(Drift::moved)
            .collect()
    }

    pub fn totals(&self) -> BTreeMap<&str, Count> {
        self.0.iter().map(|(rule, files)| (rule.as_str(), files.values().sum())).collect()
    }

    fn rule_drifts(
        rule: &str,
        found: Option<&BTreeMap<String, Count>>,
        allowed: Option<&BTreeMap<String, Count>>,
    ) -> Vec<Drift> {
        let files: BTreeSet<&String> = found.into_iter().chain(allowed).flat_map(BTreeMap::keys).collect();
        let count = |counts: Option<&BTreeMap<String, Count>>, file: &str| {
            counts.and_then(|counts| counts.get(file)).copied().unwrap_or_default()
        };
        files.into_iter().map(|file| Drift::new(rule, file, count(found, file), count(allowed, file))).collect()
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

    const ALLOWED: &str = "[no-comments]\n\"a.rs\" = 2\n\"b.rs\" = 1\n";
    const FOUND: &str = "[no-comments]\n\"a.rs\" = 3\n";
    const DRIFTS: [&str; 2] = ["no-comments a.rs: 3 found, baseline 2", "no-comments b.rs: 0 found, baseline 1"];

    #[test]
    fn drifts_name_growth_and_slack() {
        let allowed: Baseline = toml::from_str(ALLOWED).expect("valid baseline");
        let found: Baseline = toml::from_str(FOUND).expect("valid baseline");
        let drifts: Vec<String> = found.drifts(&allowed).iter().map(ToString::to_string).collect();
        assert_eq!(drifts, DRIFTS);
    }
}
