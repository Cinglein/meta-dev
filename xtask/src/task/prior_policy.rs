use std::path::Path;

use serde::Deserialize;
use task::{BaselineFile, SpecSettings, Surface};

use crate::task::revision::Revision;

#[derive(Debug, Default, Deserialize)]
pub struct PriorPolicy {
    #[serde(default)]
    surfaces: Vec<Surface>,
    baseline: Option<BaselineFile>,
    spec: Option<SpecSettings>,
}

impl PriorPolicy {
    const PATH: &str = "tooling/policy.toml";

    pub fn read_at(revision: &Revision) -> PriorPolicy {
        revision.file(Path::new(Self::PATH)).and_then(|text| toml::from_str(&text).ok()).unwrap_or_default()
    }

    pub fn surfaces(self) -> Vec<Surface> {
        self.surfaces
    }

    pub fn baseline(&self) -> Option<&BaselineFile> {
        self.baseline.as_ref()
    }

    pub fn spec(&self) -> Option<&SpecSettings> {
        self.spec.as_ref()
    }
}
