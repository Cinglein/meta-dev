mod pinned_xtask;
mod prior_budget;

use std::collections::BTreeMap;
use std::path::Path;

use meta_schema::Division;
use meta_severity::{Scale, Scales};
use serde::Deserialize;
use task::{BaselineFile, BranchName, BudgetId, Surface};

use crate::task::failure::Failure;
use crate::task::revision::{Revision, TrackedPath};
pub use pinned_xtask::PinnedXtask;
use prior_budget::PriorBudget;

#[derive(Debug, Default, Deserialize)]
pub struct PriorPolicy {
    #[serde(default)]
    surfaces: Vec<Surface>,
    baseline: Option<BaselineFile>,
    #[serde(default)]
    budgets: BTreeMap<BudgetId, PriorBudget>,
    #[serde(default)]
    scales: BTreeMap<Division, Scale>,
}

impl PriorPolicy {
    pub const PATH: &str = "tooling/policy.toml";
    pub const DEFAULT_LEVELS: &str = "crates/meta-severity/src/default_levels.json";

    pub fn read_at(revision: &Revision) -> Result<PriorPolicy, Failure> {
        Self::parsed(revision.present(Path::new(Self::PATH))?)
    }

    pub fn text_at_merge_base(root: &Path, base: &BranchName) -> Result<Option<String>, Failure> {
        Revision::merge_base(root, base)?.present(Path::new(Self::PATH))
    }

    pub fn scales_at(revision: &Revision, now: &PinnedXtask, databases: &Path) -> Result<Scales, Failure> {
        let tracked = revision.tracked(TrackedPath::new(Path::new(Self::DEFAULT_LEVELS)))?;
        let levels = Self::levels(tracked, || PinnedXtask::read_at(revision).shipped_levels(now, databases))?;
        Self::read_at(revision)?.scaled(&levels)
    }

    fn levels(tracked: Option<String>, shipped: impl FnOnce() -> Option<Scale>) -> Result<Scale, Failure> {
        tracked.map_or_else(
            || Ok(shipped().unwrap_or_else(Scale::top_default)),
            |text| serde_json::from_str(&text).map_err(Failure::PriorLevels),
        )
    }

    fn parsed(text: Option<String>) -> Result<PriorPolicy, Failure> {
        text.map_or_else(|| Ok(PriorPolicy::default()), |text| toml::from_str(&text).map_err(Failure::PriorPolicy))
    }

    fn scaled(self, defaults: &Scale) -> Result<Scales, Failure> {
        Scales::over(defaults, self.scales).map_err(Failure::PriorScales)
    }

    pub fn surfaces(self) -> Vec<Surface> {
        self.surfaces
    }

    pub fn baseline(&self) -> Option<&BaselineFile> {
        self.baseline.as_ref().or_else(|| self.budgets.values().find_map(PriorBudget::baseline))
    }

    pub fn budget_baseline(&self, budget: &BudgetId) -> Option<&BaselineFile> {
        self.budgets.get(budget).and_then(PriorBudget::baseline)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use meta_schema::Division;
    use meta_severity::Scale;

    use super::PriorPolicy;

    const UP: &str = "..";
    const NAV: &str = "features-and-maintenance/nav";
    const UNREADABLE: &str = "surfaces = 3";
    const OLD_LEVELS: &str = r#"[{"name": "blocker", "meaning": "x"}, {"name": "nuisance", "meaning": "x"}]"#;
    const SCALED: &str = "[[scales.\"features-and-maintenance/nav\"]]\nname = \"a\"\nmeaning = \"x\"\nparent = \"critical\"\n\n[[scales.\"features-and-maintenance/nav\"]]\nname = \"b\"\nmeaning = \"x\"\nparent = \"major\"\n\n[[scales.\"features-and-maintenance/nav\"]]\nname = \"c\"\nmeaning = \"x\"\nparent = \"minor\"\n";

    #[test]
    fn the_merge_base_policy_keeps_its_scales_is_absent_as_defaults_and_fails_closed_when_unreadable() {
        let nav: Division = NAV.parse().unwrap();
        let prior = PriorPolicy::parsed(Some(SCALED.to_owned())).unwrap();
        let shipped = Path::new(env!("CARGO_MANIFEST_DIR")).join(UP).join(PriorPolicy::DEFAULT_LEVELS);
        let levels: Scale = serde_json::from_str(&fs::read_to_string(shipped).unwrap()).unwrap();
        assert_eq!(prior.scaled(&levels).unwrap().owner(&nav), Some(&nav));
        assert!(PriorPolicy::parsed(None).is_ok() && PriorPolicy::parsed(Some(UNREADABLE.to_owned())).is_err());
        assert_eq!(levels, Scale::top_default());
    }

    #[test]
    fn tracked_levels_win_then_the_pinned_xtask_levels_then_the_current_defaults_and_unparseable_ones_fail_closed() {
        let old: Scale = serde_json::from_str(OLD_LEVELS).unwrap();
        let levels = |tracked: Option<&str>, shipped: Option<&Scale>| {
            PriorPolicy::levels(tracked.map(str::to_owned), || shipped.cloned()).ok()
        };
        let read = [levels(Some(OLD_LEVELS), None), levels(None, Some(&old)), levels(None, None)];
        assert_eq!(read, [Some(old.clone()), Some(old), Some(Scale::top_default())]);
        assert!(levels(Some(UNREADABLE), Some(&Scale::top_default())).is_none());
    }
}
