mod budget;
mod budget_drafts;
mod budget_id;
mod contents;
mod ledger;
mod legacy_key;
mod standing;
mod tree_failure;

use std::collections::{BTreeMap, BTreeSet};

use meta_severity::Scales;
use serde::{Deserialize, Serialize};

use super::domains::Domains;
use super::percent::Percent;
use super::policy::Most;
use budget::Budget;
pub use budget::{Amount, Measure};
pub use budget_drafts::BudgetDrafts;
use budget_id::BudgetId;
pub use contents::Contents;
pub use ledger::Ledger;
pub use legacy_key::LegacyKey;
pub use standing::{ChangeLink, Standing};
pub use tree_failure::TreeFailure;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    try_from = "BTreeMap<BudgetId, Budget>",
    into = "BTreeMap<BudgetId, Budget>"
)]
pub struct Budgets(BTreeMap<BudgetId, Budget>);

impl Budgets {
    #[must_use]
    pub fn budget(&self, id: &BudgetId) -> Option<&Budget> {
        self.0.get(id)
    }

    #[must_use]
    pub fn whole(&self, measure: &Measure) -> Option<&Budget> {
        self.0
            .values()
            .find(|budget| budget.top_measure().as_ref() == Some(measure))
    }

    #[must_use]
    pub fn blocking_line(&self, measure: &Measure) -> Option<Amount> {
        self.whole(measure).and_then(Budget::blocking_line)
    }

    #[must_use]
    pub fn amount(&self, budget: &Budget) -> Option<Amount> {
        budget
            .allowance()
            .resolved(|parent| self.0.get(parent).and_then(|parent| self.amount(parent)))
    }

    #[must_use]
    pub fn standing(&self) -> bool {
        self.0.values().any(|budget| budget.standing_in().is_some())
    }

    fn split(&self, parent: &BudgetId) -> Result<(), TreeFailure> {
        let oversubscribed = || Err(TreeFailure::Oversubscribed(parent.clone()));
        let shares = Percent::total(
            self.0
                .values()
                .filter_map(|budget| budget.allowance().share_of(parent)),
        );
        let counts: Vec<Most> = self
            .0
            .values()
            .filter_map(|budget| budget.allowance().count_of(parent))
            .collect();
        let counted = counts
            .iter()
            .try_fold(Most::from(0), |sum, count| sum.plus(*count));
        let whole = self
            .0
            .get(parent)
            .and_then(|budget| self.amount(budget))
            .and_then(|amount| amount.count());
        match (shares, counted) {
            (None, _) | (_, None) => oversubscribed(),
            (Some(shares), _) if shares.is_more_than_whole() => oversubscribed(),
            (Some(shares), _) if shares > Percent::NONE && !counts.is_empty() => {
                Err(TreeFailure::MixedParts(parent.clone()))
            }
            (_, Some(counted)) if whole.map_or(!counts.is_empty(), |whole| counted > whole) => {
                oversubscribed()
            }
            (Some(_), Some(_)) => Ok(()),
        }
    }

    pub fn measurable(&self, domains: &Domains) -> Result<(), TreeFailure> {
        self.0
            .iter()
            .try_for_each(|(id, budget)| budget.measurable(id, domains))
    }

    pub fn leveled(&self, scales: &Scales) -> Result<(), TreeFailure> {
        self.0
            .iter()
            .try_for_each(|(id, budget)| budget.leveled(id, scales))
    }
}

impl TryFrom<BTreeMap<BudgetId, Budget>> for Budgets {
    type Error = TreeFailure;

    fn try_from(budgets: BTreeMap<BudgetId, Budget>) -> Result<Budgets, TreeFailure> {
        budgets
            .iter()
            .try_for_each(|(id, budget)| budget.placed(id, &budgets))?;
        let mut measured = BTreeSet::new();
        let duplicate = budgets.iter().find_map(|(id, budget)| {
            budget
                .top_measure()
                .filter(|measure| {
                    !measured.insert((measure.clone(), budget.standing_in().cloned()))
                })
                .map(|_| id.clone())
        });
        let tree = Budgets(budgets);
        match (
            duplicate,
            tree.0.keys().find_map(|parent| tree.split(parent).err()),
        ) {
            (Some(id), _) => Err(TreeFailure::Duplicate(id)),
            (None, Some(failure)) => Err(failure),
            (None, None) => Ok(tree),
        }
    }
}

impl From<Budgets> for BTreeMap<BudgetId, Budget> {
    fn from(budgets: Budgets) -> BTreeMap<BudgetId, Budget> {
        budgets.0
    }
}

#[cfg(test)]
mod tests {
    use super::Budgets;
    use super::budget::State;

    const TREE: &str = r#"
[ci]
division = "performance-and-cost"
measure = { workflow_wall_time = "diff_job" }
allowance = { duration = 4000 }
window = "per_change"
source = "ci"

[ci-tests]
division = "performance-and-cost/tests"
measure = "test_wall_time"
allowance = { of = "ci", percent = 60 }
window = "per_change"
source = "ci"
overspend = "block"

[ci-lints]
division = "performance-and-cost/lints"
measure = "slowest_test"
allowance = { of = "ci", percent = 40 }
window = "per_change"
source = "ci"
warning = 50
tolerance = 10
"#;
    const WRITTEN: &str = "[ci]\ndivision = \"performance-and-cost\"\nwindow = \"per_change\"\nsource = \"ci\"\nwarning = 60\noverspend = \"block\"\n\n[ci.measure]\nworkflow_wall_time = \"diff_job\"\n\n[ci.allowance]\nduration = 4000\n\n[ci-lints]\ndivision = \"performance-and-cost/lints\"\nmeasure = \"slowest_test\"\nwindow = \"per_change\"\nsource = \"ci\"\nwarning = 50\ntolerance = 10\noverspend = \"block\"\n\n[ci-lints.allowance]\nof = \"ci\"\npercent = 40\n\n[ci-tests]\ndivision = \"performance-and-cost/tests\"\nmeasure = \"test_wall_time\"\nwindow = \"per_change\"\nsource = \"ci\"\nwarning = 60\noverspend = \"block\"\n\n[ci-tests.allowance]\nof = \"ci\"\npercent = 60\n";
    const WINDOW: &str = "window = ";
    const MEASURED: &str = "measure = \"tangle_items\"\nsource = \"ci\"\nwindow = ";
    const PART: &str = "ci-tests";
    const WHOLE: &str = "ci";
    const PART_AMOUNT: &str = "2400ms";
    const OVER: Option<State> = Some(State::Over);
    const WARN: Option<State> = Some(State::Warn);
    const UNITLESS: [&str; 3] = [
        "[ci]\ndivision = \"performance-and-cost\"\nwindow = \"standing\"\n",
        "[ci]\ndivision = \"performance-and-cost\"\nallowance = {}\nwindow = \"standing\"\n",
        "[ci]\ndivision = \"performance-and-cost\"\nallowance = { count = 1, duration = 2 }\nwindow = \"standing\"\n",
    ];
    const STANDING: &str = "\"standing\"";
    const PER_CHANGE: &str = "\"per_change\"";
    const REFUSED: [(&str, &str); 6] = [
        (
            "[orphan]\ndivision = \"performance-and-cost/tests\"\nallowance = { of = \"ci\", percent = 5 }\nwindow = \"standing\"\n",
            "budget orphan: part of a budget that is not defined",
        ),
        (
            "[deep]\ndivision = \"performance-and-cost/tests\"\nallowance = { count = 9 }\nwindow = \"standing\"\n",
            "budget deep: a budget in a nested division is a part of its parent division's budget",
        ),
        (
            "[ci]\ndivision = \"performance-and-cost\"\nallowance = { count = 9 }\nwindow = \"standing\"\n\n[a]\ndivision = \"performance-and-cost/a\"\nallowance = { of = \"ci\", percent = 9223372036854775807 }\nwindow = \"standing\"\n\n[b]\ndivision = \"performance-and-cost/b\"\nallowance = { of = \"ci\", percent = 9223372036854775807 }\nwindow = \"standing\"\n\n[c]\ndivision = \"performance-and-cost/c\"\nallowance = { of = \"ci\", percent = 9223372036854775807 }\nwindow = \"standing\"\n",
            "budget ci: its parts add up to more than all of it",
        ),
        (
            "[ci]\ndivision = \"performance-and-cost\"\nallowance = { count = 9 }\nwindow = \"standing\"\n\n[outside]\ndivision = \"features-and-maintenance/tests\"\nallowance = { of = \"ci\", percent = 5 }\nwindow = \"standing\"\n",
            "budget outside: its division is not inside the division of the budget it is part of",
        ),
        (
            "[ci]\ndivision = \"performance-and-cost\"\nallowance = { count = 9 }\nwindow = \"standing\"\n\n[larger]\ndivision = \"performance-and-cost/tests\"\nallowance = { of = \"ci\", percent = 101 }\nwindow = \"standing\"\n",
            "budget ci: its parts add up to more than all of it",
        ),
        (
            "[ci]\ndivision = \"performance-and-cost\"\nallowance = { count = 50 }\nwindow = \"standing\"\n\n[a]\ndivision = \"performance-and-cost/a\"\nallowance = { of = \"ci\", percent = 51 }\nwindow = \"standing\"\n\n[b]\ndivision = \"performance-and-cost/b\"\nallowance = { of = \"ci\", percent = 50 }\nwindow = \"standing\"\n",
            "budget ci: its parts add up to more than all of it",
        ),
    ];

    #[test]
    fn a_budget_reads_back_with_its_default_overspend_and_a_part_keeps_its_share() {
        let budgets: Budgets = toml::from_str(TREE).unwrap();
        assert_eq!(toml::to_string(&budgets).unwrap(), WRITTEN);
        let [part, whole] = [PART, WHOLE].map(|id| budgets.budget(&id.parse().unwrap()).unwrap());
        assert!(part.blocks(OVER) && whole.blocks(OVER) && !whole.blocks(WARN));
        assert_eq!(
            budgets
                .amount(part)
                .map(|amount| amount.to_string())
                .as_deref(),
            Some(PART_AMOUNT)
        );
    }

    impl Budgets {
        fn measured(tree: &str) -> Result<Budgets, toml::de::Error> {
            toml::from_str(&tree.replace(STANDING, PER_CHANGE).replace(WINDOW, MEASURED))
        }
    }

    #[test]
    fn a_budget_without_exactly_one_unit_does_not_parse() {
        assert!(UNITLESS.iter().all(|tree| Budgets::measured(tree).is_err()));
    }

    #[test]
    fn a_part_names_a_defined_parent_sits_inside_its_division_and_fits_its_share() {
        let refusals = REFUSED.map(|(tree, _)| {
            Budgets::measured(tree)
                .err()
                .map(|error| error.message().to_owned())
        });
        assert_eq!(
            refusals,
            REFUSED.map(|(_, refusal)| Some(refusal.to_owned()))
        );
    }
}
