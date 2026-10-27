use std::path::Path;

use meta_severity::Scales;
use serde::Deserialize;

use crate::request::cli_failure::{CliFailure, Setup};

#[derive(Debug, Default, Deserialize)]
pub struct ScalePolicy {
    #[serde(default)]
    scales: Scales,
}

impl ScalePolicy {
    const FILE: &'static str = "tooling/policy.toml";

    pub fn read(root: &Path) -> Result<Scales, CliFailure> {
        let read = std::fs::read_to_string(root.join(Self::FILE)).map_or_else(
            |_| Ok(ScalePolicy::default()),
            |text| toml::from_str(&text).map_err(CliFailure::raised::<Setup>),
        );
        read.map(|policy| policy.scales)
    }
}

#[cfg(test)]
mod tests;
