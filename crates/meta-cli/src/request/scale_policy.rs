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
pub use shell_rules::ShellRules;

mod liveness;
mod shell_rules;

#[derive(Debug, Default, Deserialize)]
pub struct ScalePolicy {
    #[serde(default)]
    scales: Scales,
    #[serde(default)]
    domains: BTreeMap<Division, Option<IgnoredAny>>,
    #[serde(default)]
    liveness: Liveness,
    #[serde(default)]
    shell: Option<ShellRules>,
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

    pub fn shell_in(root: Option<&Path>) -> Result<ShellRules, CliFailure> {
        let own = root.filter(|root| root.join(Self::FILE).exists()).map(Self::read).transpose()?;
        let embedded = || toml::from_str::<ScalePolicy>(Self::EMBEDDED).map_err(CliFailure::raised::<Setup>);
        own.and_then(|policy| policy.shell)
            .map_or_else(|| embedded().map(|policy| policy.shell.unwrap_or_default()), Ok)
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
        ScalePolicy { scales, domains: BTreeMap::new(), liveness: Liveness::default(), shell: None }
    }
}

#[cfg(test)]
mod tests;
