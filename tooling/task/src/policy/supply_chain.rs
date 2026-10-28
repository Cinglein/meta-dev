mod duplicate_level;

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::crate_name::CrateName;
use duplicate_level::DuplicateLevel;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SupplyChain {
    licenses: Vec<String>,
    registries: Vec<String>,
    git: Vec<String>,
    ignored_advisories: Vec<String>,
    duplicates: DuplicateLevel,
    semver: Vec<CrateName>,
}

impl SupplyChain {
    #[must_use]
    pub fn semver(&self) -> &[CrateName] {
        &self.semver
    }
}

impl fmt::Display for SupplyChain {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let listed = |entries: &Vec<String>| toml::Value::from(entries.clone());
        write!(
            formatter,
            "[graph]\nall-features = true\n\n[advisories]\nignore = {}\n\n[licenses]\nallow = {}\n\n[licenses.private]\nignore = true\n\n[bans]\nmultiple-versions = \"{}\"\n\n[sources]\nunknown-registry = \"deny\"\nunknown-git = \"deny\"\nallow-registry = {}\nallow-git = {}\n",
            listed(&self.ignored_advisories),
            listed(&self.licenses),
            self.duplicates,
            listed(&self.registries),
            listed(&self.git),
        )
    }
}
