mod budget;
mod budget_id;
mod tree_failure;

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::percent::Percent;
use budget::Budget;
use budget_id::BudgetId;
use tree_failure::TreeFailure;

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
}

impl TryFrom<BTreeMap<BudgetId, Budget>> for Budgets {
    type Error = TreeFailure;

    fn try_from(budgets: BTreeMap<BudgetId, Budget>) -> Result<Budgets, TreeFailure> {
        budgets
            .iter()
            .try_for_each(|(id, budget)| budget.placed(id, &budgets))?;
        let shared = |parent: &BudgetId| {
            Percent::total(
                budgets
                    .values()
                    .filter_map(|budget| budget.allowance().share_of(parent)),
            )
        };
        match budgets
            .keys()
            .find(|parent| shared(parent).is_none_or(Percent::is_more_than_whole))
        {
            Some(parent) => Err(TreeFailure::Oversubscribed(parent.clone())),
            None => Ok(Budgets(budgets)),
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
    use super::budget::Overspend;

    const TREE: &str = r#"
[ci]
division = "performance-and-cost"
allowance = { duration = 4000 }
window = { rolling = 604800000 }

[ci-tests]
division = "performance-and-cost/tests"
allowance = { of = "ci", percent = 60 }
window = { rolling = 604800000 }
overspend = "escalate"

[ci-lints]
division = "performance-and-cost/lints"
allowance = { of = "ci", percent = 40 }
window = { rolling = 604800000 }
"#;
    const WRITTEN: &str = "[ci]\ndivision = \"performance-and-cost\"\noverspend = \"block\"\n\n[ci.allowance]\nduration = 4000\n\n[ci.window]\nrolling = 604800000\n\n[ci-lints]\ndivision = \"performance-and-cost/lints\"\noverspend = \"block\"\n\n[ci-lints.allowance]\nof = \"ci\"\npercent = 40\n\n[ci-lints.window]\nrolling = 604800000\n\n[ci-tests]\ndivision = \"performance-and-cost/tests\"\noverspend = \"escalate\"\n\n[ci-tests.allowance]\nof = \"ci\"\npercent = 60\n\n[ci-tests.window]\nrolling = 604800000\n";
    const PART: &str = "ci-tests";
    const SHARE: usize = 60;
    const ESCALATE: Overspend = Overspend::Escalate;
    const UNITLESS: [&str; 3] = [
        "[ci]\ndivision = \"performance-and-cost\"\nwindow = \"standing\"\n",
        "[ci]\ndivision = \"performance-and-cost\"\nallowance = {}\nwindow = \"standing\"\n",
        "[ci]\ndivision = \"performance-and-cost\"\nallowance = { count = 1, duration = 2 }\nwindow = \"standing\"\n",
    ];
    const REFUSED: [(&str, &str); 7] = [
        (
            "[orphan]\ndivision = \"performance-and-cost/tests\"\nallowance = { of = \"ci\", percent = 5 }\nwindow = \"standing\"\n",
            "budget orphan: part of a budget that is not defined",
        ),
        (
            "[deep]\ndivision = \"performance-and-cost/tests\"\nallowance = { count = 9 }\nwindow = \"standing\"\n",
            "budget deep: a budget in a nested division is a part of its parent division's budget",
        ),
        (
            "[ci]\ndivision = \"performance-and-cost\"\nallowance = { count = 9 }\nwindow = \"standing\"\n\n[other]\ndivision = \"performance-and-cost/tests\"\nallowance = { of = \"ci\", percent = 5 }\nwindow = \"per_change\"\n",
            "budget other: a part has the window of the budget it is part of",
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
            "[ci]\ndivision = \"performance-and-cost\"\nallowance = { share = 50 }\nwindow = \"standing\"\n\n[a]\ndivision = \"performance-and-cost/a\"\nallowance = { of = \"ci\", percent = 51 }\nwindow = \"standing\"\n\n[b]\ndivision = \"performance-and-cost/b\"\nallowance = { of = \"ci\", percent = 50 }\nwindow = \"standing\"\n",
            "budget ci: its parts add up to more than all of it",
        ),
    ];

    #[test]
    fn a_budget_reads_back_with_its_default_overspend_and_a_part_keeps_its_share() {
        let budgets: Budgets = toml::from_str(TREE).unwrap();
        assert_eq!(toml::to_string(&budgets).unwrap(), WRITTEN);
        let part = budgets.budget(&PART.parse().unwrap()).unwrap();
        assert!(
            part.overspend() == ESCALATE
                && part
                    .allowance()
                    .part()
                    .is_some_and(|share| share.percent() == SHARE.into())
        );
    }

    #[test]
    fn a_budget_without_exactly_one_unit_does_not_parse() {
        assert!(
            UNITLESS
                .iter()
                .all(|tree| toml::from_str::<Budgets>(tree).is_err())
        );
    }

    #[test]
    fn a_part_names_a_defined_parent_sits_inside_its_division_and_fits_its_share() {
        let refusals = REFUSED.map(|(tree, _)| {
            toml::from_str::<Budgets>(tree)
                .err()
                .map(|error| error.message().to_owned())
        });
        assert_eq!(
            refusals,
            REFUSED.map(|(_, refusal)| Some(refusal.to_owned()))
        );
    }
}
