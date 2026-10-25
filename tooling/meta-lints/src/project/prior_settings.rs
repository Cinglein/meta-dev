use std::path::Path;

use serde::Deserialize;

use crate::failure::Failure;
use crate::prior::Revision;
use crate::project::prior_lints::PriorLints;
use crate::project::surfaces::Surfaces;

#[derive(Clone, Debug, Deserialize)]
pub struct PriorSettings {
    #[serde(default)]
    surfaces: Surfaces,
    lints: PriorLints,
}

impl PriorSettings {
    pub fn read_at(revision: &Revision, config: &Path) -> Result<Option<PriorSettings>, Failure> {
        let prior = revision.file(config)?;
        Ok(prior.and_then(|text| toml::from_str::<PriorSettings>(&text).ok()))
    }

    pub fn lints(self) -> PriorLints {
        self.lints
    }

    pub fn surfaces(self) -> Surfaces {
        self.surfaces
    }
}
