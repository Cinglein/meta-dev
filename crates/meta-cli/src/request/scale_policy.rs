use std::collections::BTreeMap;
use std::io::ErrorKind;
use std::path::Path;

use meta_schema::Division;
use meta_severity::{RatingFailure, Scales};
use serde::Deserialize;
use serde::de::IgnoredAny;

use crate::request::cli_failure::{CliFailure, Setup};
use crate::request::local::LocalFile;

#[derive(Debug, Default, Deserialize)]
pub struct ScalePolicy {
    #[serde(default)]
    scales: Scales,
    #[serde(default)]
    domains: BTreeMap<Division, Option<IgnoredAny>>,
}

impl ScalePolicy {
    const FILE: &'static str = "tooling/policy.toml";
    const MISSING: &'static str =
        "missing; scales are not defaulted; copy meta-dev's tooling/policy.toml to it (meta init will do this later)";

    pub fn read(root: &Path) -> Result<ScalePolicy, CliFailure> {
        let file = LocalFile::new(root.join(Self::FILE));
        let text = std::fs::read_to_string(root.join(Self::FILE)).map_err(|failure| match failure.kind() {
            ErrorKind::NotFound => file.failure(Self::MISSING),
            _ => file.failure(failure),
        })?;
        toml::from_str(&text).map_err(CliFailure::raised::<Setup>)
    }

    pub fn scales(&self) -> &Scales {
        &self.scales
    }

    pub fn known(&self, division: &Division) -> Result<(), RatingFailure> {
        (self.scales.declares(division) || self.domains.contains_key(division))
            .then_some(())
            .ok_or_else(|| RatingFailure::Undeclared(division.clone()))
    }
}

impl From<Scales> for ScalePolicy {
    fn from(scales: Scales) -> ScalePolicy {
        ScalePolicy { scales, domains: BTreeMap::new() }
    }
}

#[cfg(test)]
mod tests;
