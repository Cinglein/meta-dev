use serde::Deserialize;

use crate::lint_config::LintConfig;
use crate::project::surfaces::Surfaces;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    surfaces: Surfaces,
    lints: LintConfig,
}

impl Settings {
    pub fn surfaces(&self) -> &Surfaces {
        &self.surfaces
    }

    pub fn lints(&self) -> &LintConfig {
        &self.lints
    }
}
