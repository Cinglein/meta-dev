use std::collections::BTreeMap;
use std::path::Path;

use meta_schema::Division;
use meta_severity::{RatingFailure, Scales};
use serde::Deserialize;
use serde::de::IgnoredAny;

use crate::request::cli_failure::{CliFailure, Setup};
pub use liveness::Liveness;

mod liveness;

#[derive(Debug, Default, Deserialize)]
pub struct ScalePolicy {
    #[serde(default)]
    scales: Scales,
    #[serde(default)]
    domains: BTreeMap<Division, Option<IgnoredAny>>,
    #[serde(default)]
    liveness: Liveness,
}

impl ScalePolicy {
    const FILE: &'static str = "tooling/policy.toml";

    pub fn read(root: &Path) -> Result<ScalePolicy, CliFailure> {
        std::fs::read_to_string(root.join(Self::FILE)).map_or_else(
            |_| Ok(ScalePolicy::default()),
            |text| toml::from_str(&text).map_err(CliFailure::raised::<Setup>),
        )
    }

    pub fn scales(&self) -> &Scales {
        &self.scales
    }

    pub fn liveness(&self) -> Liveness {
        self.liveness
    }

    pub fn known(&self, division: &Division) -> Result<(), RatingFailure> {
        (self.scales.declares(division) || self.domains.contains_key(division))
            .then_some(())
            .ok_or_else(|| RatingFailure::Undeclared(division.clone()))
    }
}

impl From<Scales> for ScalePolicy {
    fn from(scales: Scales) -> ScalePolicy {
        ScalePolicy { scales, domains: BTreeMap::new(), liveness: Liveness::default() }
    }
}

#[cfg(test)]
mod tests;
