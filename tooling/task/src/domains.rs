mod domain;
mod domain_failure;
mod extent;
mod scope_kind;

use std::collections::BTreeMap;

use meta_schema::{Division, TopDivision};
use serde::{Deserialize, Serialize};

pub use domain::Domain;
pub use domain_failure::DomainFailure;
pub use extent::Extent;
pub use scope_kind::ScopeKind;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    try_from = "BTreeMap<Division, Domain>",
    into = "BTreeMap<Division, Domain>"
)]
pub struct Domains(BTreeMap<Division, Domain>);

impl Domains {
    const TOPS: [TopDivision; 3] = [
        TopDivision::FeaturesAndMaintenance,
        TopDivision::PerformanceAndCost,
        TopDivision::ArchitectureAndPlanning,
    ];

    #[must_use]
    pub fn domain(&self, division: &Division) -> Option<&Domain> {
        self.0.get(division)
    }

    fn placed(&self, division: &Division, domain: &Domain) -> Result<(), DomainFailure> {
        let planning = Division::from(TopDivision::ArchitectureAndPlanning);
        let lineage = division.lineage();
        match lineage
            .iter()
            .rev()
            .skip(1)
            .find_map(|ancestor| self.0.get(ancestor))
        {
            _ if lineage.first() == Some(&planning) && !domain.is_empty() => {
                Err(DomainFailure::PlanningCode(division.clone()))
            }
            Some(parent) if !domain.is_strictly_within(parent) => {
                Err(DomainFailure::Outside(division.clone()))
            }
            Some(_) | None => Ok(()),
        }
    }
}

impl Default for Domains {
    fn default() -> Domains {
        Domains(
            Self::TOPS
                .into_iter()
                .map(|top| (Division::from(top), Domain::owned_by(top)))
                .collect(),
        )
    }
}

impl TryFrom<BTreeMap<Division, Domain>> for Domains {
    type Error = DomainFailure;

    fn try_from(declared: BTreeMap<Division, Domain>) -> Result<Domains, DomainFailure> {
        let mut domains = Domains::default();
        domains.0.extend(declared);
        domains
            .0
            .iter()
            .try_for_each(|(division, domain)| domains.placed(division, domain))?;
        Ok(domains)
    }
}

impl From<Domains> for BTreeMap<Division, Domain> {
    fn from(domains: Domains) -> BTreeMap<Division, Domain> {
        domains.0
    }
}

#[cfg(test)]
mod tests {
    use meta_schema::{Division, TopDivision};
    use settings::EmbeddedDefaults;
    use strum::IntoEnumIterator;

    use super::{Domains, Extent, ScopeKind};
    use crate::policy::Policy;

    const TOPS: [TopDivision; 3] = [
        TopDivision::PerformanceAndCost,
        TopDivision::FeaturesAndMaintenance,
        TopDivision::ArchitectureAndPlanning,
    ];
    const DECLARED: [&[ScopeKind]; 3] = [
        &[ScopeKind::Tests, ScopeKind::Workflows],
        &[ScopeKind::Crates, ScopeKind::Paths],
        &[],
    ];
    const NESTED_DOMAIN: &str = "[domains.\"performance-and-cost/unit\"]\ntests = [\"*\"]\n";
    const NESTED: &str = "\n[domains.\"performance-and-cost/unit\"]\ntests = [\"*\"]\n\n[budgets.unit-time]\ndivision = \"performance-and-cost/unit\"\nmeasure = \"test_wall_time\"\nallowance = { of = \"test-time\", percent = 50 }\nwindow = \"per_change\"\nsource = \"local_gate\"\nwarning = 50\ntolerance = 10\n";
    const REFUSED: [(&str, &str); 7] = [
        (
            "\n[budgets.whole]\ndivision = \"performance-and-cost\"\nmeasure = \"test_wall_time\"\nallowance = { duration = 4000 }\nwindow = \"per_change\"\nsource = \"ci\"\n\n[budgets.part]\ndivision = \"performance-and-cost/unit\"\nmeasure = \"tangle_items\"\nallowance = { of = \"whole\", percent = 50 }\nwindow = \"per_change\"\nsource = \"ci\"\n",
            "budget part: its allowance is not in its measure's unit",
        ),
        (
            "\n[domains.\"performance-and-cost/unit\"]\ntests = [\"*\"]\nworkflows = [\"*\"]\n",
            "domain performance-and-cost/unit: not strictly inside its parent's domain",
        ),
        (
            "\n[domains.\"performance-and-cost/unit\"]\ncrates = [\"meta-cli\"]\n",
            "domain performance-and-cost/unit: not strictly inside its parent's domain",
        ),
        (
            "\n[domains.architecture-and-planning]\npaths = [\"context*\"]\n",
            "domain architecture-and-planning: planning has no code scopes",
        ),
        (
            "\n[budgets.lines]\ndivision = \"performance-and-cost/lines\"\nmeasure = \"test_wall_time\"\nallowance = { of = \"test-time\", percent = 50 }\nwindow = \"per_change\"\nsource = \"local_gate\"\n",
            "budget lines: its measure needs tests, which its division's domain does not declare",
        ),
        (
            "\n[budgets.time]\ndivision = \"performance-and-cost\"\nmeasure = \"test_wall_time\"\nallowance = { count = 4 }\nwindow = \"per_change\"\nsource = \"ci\"\n",
            "budget time: its allowance is not in its measure's unit",
        ),
        (
            "\n[budgets.time]\ndivision = \"performance-and-cost\"\nmeasure = \"test_wall_time\"\nallowance = { duration = 4000 }\nwindow = \"per_change\"\nsource = \"ci\"\nwarning = 100\n",
            "warning line 100: must be below the blocking line at 100",
        ),
    ];

    impl Policy {
        fn with_added(added: &str) -> Result<Policy, String> {
            toml::from_str::<Policy>(&format!("{}{added}", Policy::DEFAULTS))
                .map_err(|error| error.message().to_owned())
        }
    }

    #[test]
    fn by_default_ci_owns_tests_and_workflows_code_owns_crates_and_paths_and_planning_nothing() {
        let domains = Domains::default();
        let declared = TOPS.map(|top| {
            let domain = domains
                .domain(&Division::from(top))
                .cloned()
                .unwrap_or_default();
            ScopeKind::iter()
                .filter(|kind| domain.extent(*kind) == Extent::Whole)
                .collect::<Vec<ScopeKind>>()
        });
        assert_eq!(declared, DECLARED.map(<[ScopeKind]>::to_vec));
    }

    #[test]
    fn a_nested_domain_inside_its_parent_and_a_budget_on_a_declared_scope_are_read() {
        let nested = Policy::with_added(NESTED).unwrap();
        let written = toml::to_string(&nested).unwrap();
        assert_eq!(toml::from_str::<Policy>(&written).ok(), Some(nested));
        assert!(written.contains(NESTED_DOMAIN));
    }

    #[test]
    fn domains_outside_their_parent_planning_code_and_unmeasurable_budgets_are_refused() {
        let refusals = REFUSED.map(|(added, _)| Policy::with_added(added).err());
        assert_eq!(
            refusals,
            REFUSED.map(|(_, refusal)| Some(refusal.to_owned()))
        );
    }
}
