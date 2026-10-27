mod prior_budget;

use std::collections::BTreeMap;
use std::path::Path;

use serde::Deserialize;
use task::{BaselineFile, Surface};

use crate::task::revision::Revision;
use prior_budget::PriorBudget;

#[derive(Debug, Default, Deserialize)]
pub struct PriorPolicy {
    #[serde(default)]
    surfaces: Vec<Surface>,
    baseline: Option<BaselineFile>,
    #[serde(default)]
    budgets: BTreeMap<String, PriorBudget>,
}

impl PriorPolicy {
    pub const PATH: &str = "tooling/policy.toml";

    pub fn read_at(revision: &Revision) -> PriorPolicy {
        revision.file(Path::new(Self::PATH)).and_then(|text| toml::from_str(&text).ok()).unwrap_or_default()
    }

    pub fn surfaces(self) -> Vec<Surface> {
        self.surfaces
    }

    pub fn baseline(&self) -> Option<&BaselineFile> {
        self.baseline.as_ref().or_else(|| self.budgets.values().find_map(PriorBudget::baseline))
    }
}
