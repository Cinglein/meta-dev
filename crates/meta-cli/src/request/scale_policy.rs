use std::collections::BTreeMap;
use std::io::ErrorKind;
use std::path::Path;

use meta_schema::Division;
use meta_severity::{RatingFailure, Scales};
use serde::Deserialize;
use serde::de::IgnoredAny;

use crate::request::cli_failure::{CliFailure, Setup};
use crate::request::local::LocalFile;
pub use liveness::Liveness;
pub use policy_millis::PolicyMillis;
pub use queue_policy::QueuePolicy;
pub use tool_policy::ToolPolicy;

mod liveness;
mod policy_millis;
mod queue_policy;
mod tool_policy;

#[derive(Debug, Default, Deserialize)]
pub struct ScalePolicy {
    #[serde(default)]
    scales: Scales,
    #[serde(default)]
    domains: BTreeMap<Division, Option<IgnoredAny>>,
    #[serde(default)]
    millis: PolicyMillis,
    #[serde(default)]
    liveness: Liveness,
    #[serde(default)]
    queue: QueuePolicy,
    #[serde(default)]
    tools: ToolPolicy,
}

impl ScalePolicy {
    const FILE: &'static str = "tooling/policy.toml";
    const EMBEDDED: &'static str = include_str!("../../../../tooling/policy.toml");
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

    pub fn millis(&self) -> PolicyMillis {
        self.millis
    }

    pub fn queue(&self) -> &QueuePolicy {
        &self.queue
    }

    pub fn tools(&self) -> ToolPolicy {
        let embedded = toml::from_str::<ScalePolicy>(Self::EMBEDDED).map(|policy| policy.tools).unwrap_or_default();
        embedded.union(self.tools.clone())
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
        ScalePolicy { scales, ..ScalePolicy::default() }
    }
}

#[cfg(test)]
mod tests;
