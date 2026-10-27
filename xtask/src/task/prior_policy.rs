mod prior_budget;

use std::collections::BTreeMap;
use std::path::Path;

use meta_severity::Scales;
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
    #[serde(default)]
    scales: Scales,
}

impl PriorPolicy {
    pub const PATH: &str = "tooling/policy.toml";

    pub fn read_at(revision: &Revision) -> PriorPolicy {
        revision.file(Path::new(Self::PATH)).and_then(|text| toml::from_str(&text).ok()).unwrap_or_default()
    }

    pub fn scales(self) -> Scales {
        self.scales
    }

    pub fn surfaces(self) -> Vec<Surface> {
        self.surfaces
    }

    pub fn baseline(&self) -> Option<&BaselineFile> {
        self.baseline.as_ref().or_else(|| self.budgets.values().find_map(PriorBudget::baseline))
    }
}

#[cfg(test)]
mod tests {
    use meta_schema::Division;

    use super::PriorPolicy;

    const NAV: &str = "features-and-maintenance/nav";
    const SCALED: &str = "[[scales.\"features-and-maintenance/nav\"]]\nname = \"a\"\nmeaning = \"x\"\nparent = \"critical\"\n\n[[scales.\"features-and-maintenance/nav\"]]\nname = \"b\"\nmeaning = \"x\"\nparent = \"major\"\n\n[[scales.\"features-and-maintenance/nav\"]]\nname = \"c\"\nmeaning = \"x\"\nparent = \"minor\"\n";

    #[test]
    fn the_merge_base_policy_keeps_its_scales() {
        let nav: Division = NAV.parse().unwrap();
        let prior: PriorPolicy = toml::from_str(SCALED).unwrap();
        assert_eq!(prior.scales().owner(&nav), Some(&nav));
    }
}
