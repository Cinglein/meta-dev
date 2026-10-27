mod prior_budget;

use std::collections::BTreeMap;
use std::path::Path;

use meta_severity::Scales;
use serde::Deserialize;
use task::{BaselineFile, Surface};

use crate::task::failure::Failure;
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

    pub fn read_at(revision: &Revision) -> Result<PriorPolicy, Failure> {
        Self::parsed(revision.file(Path::new(Self::PATH)))
    }

    fn parsed(text: Option<String>) -> Result<PriorPolicy, Failure> {
        text.map_or_else(|| Ok(PriorPolicy::default()), |text| toml::from_str(&text).map_err(Failure::PriorPolicy))
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
    const UNREADABLE: &str = "surfaces = 3";
    const SCALED: &str = "[[scales.\"features-and-maintenance/nav\"]]\nname = \"a\"\nmeaning = \"x\"\nparent = \"critical\"\n\n[[scales.\"features-and-maintenance/nav\"]]\nname = \"b\"\nmeaning = \"x\"\nparent = \"major\"\n\n[[scales.\"features-and-maintenance/nav\"]]\nname = \"c\"\nmeaning = \"x\"\nparent = \"minor\"\n";

    #[test]
    fn the_merge_base_policy_keeps_its_scales_is_absent_as_defaults_and_fails_closed_when_unreadable() {
        let nav: Division = NAV.parse().unwrap();
        let prior = PriorPolicy::parsed(Some(SCALED.to_owned())).unwrap();
        assert_eq!(prior.scales().owner(&nav), Some(&nav));
        assert!(PriorPolicy::parsed(None).is_ok() && PriorPolicy::parsed(Some(UNREADABLE.to_owned())).is_err());
    }
}
