use std::path::Path;

use serde::Deserialize;
use task::{BaselineFile, Policy, Surface};

use crate::task::revision::Revision;

#[derive(Debug, Default, Deserialize)]
pub struct PriorPolicy {
    #[serde(default)]
    surfaces: Vec<Surface>,
    baseline: Option<BaselineFile>,
}

impl PriorPolicy {
    pub const PATH: &str = "tooling/policy.toml";

    pub fn read_at(revision: &Revision) -> PriorPolicy {
        let text = revision.file(Path::new(Self::PATH));
        let prior: PriorPolicy = text.as_deref().and_then(|text| toml::from_str(text).ok()).unwrap_or_default();
        let typed = text.as_deref().and_then(|text| toml::from_str::<Policy>(text).ok());
        PriorPolicy { baseline: typed.map(|policy| policy.baseline().clone()).or(prior.baseline), ..prior }
    }

    pub fn surfaces(self) -> Vec<Surface> {
        self.surfaces
    }

    pub fn baseline(&self) -> Option<&BaselineFile> {
        self.baseline.as_ref()
    }
}
