mod row;
mod statement;

use std::collections::BTreeMap;

use super::budget::{Amount, Budget, Measure};
use super::standing::{Exposure, Standing};
use crate::failure::Failure;
use crate::policy::Policy;
use crate::selection::Selection;
use row::Row;
use statement::Statement;

#[derive(Debug, Default)]
pub struct Ledger {
    spent: BTreeMap<Measure, Amount>,
    standing: Option<Standing>,
}

impl Ledger {
    pub fn spend(&mut self, measure: Measure, amount: Amount) {
        self.spent.insert(measure, amount);
    }

    pub fn count(&mut self, standing: Option<Standing>) {
        self.standing = standing;
    }

    #[must_use]
    pub fn spend_of(&self, budget: &Budget) -> Option<Amount> {
        budget.measure().item_filter().map_or_else(
            || self.spent.get(&budget.measure()).cloned(),
            |filter| {
                self.standing
                    .as_ref()
                    .map(|standing| standing.spend(filter, budget.division()))
            },
        )
    }

    #[must_use]
    pub fn exposure_of(&self, budget: &Budget) -> Exposure {
        budget
            .measure()
            .item_filter()
            .zip(self.standing.as_ref())
            .map_or(Exposure::Diff, |(filter, standing)| {
                standing.exposure(filter, budget.division())
            })
    }

    pub fn checked(&self, policy: &Policy, selection: &Selection) -> Result<(), Failure> {
        let statement = self.statement(policy);
        print!("{statement}");
        statement.verdict(selection).verdict()
    }

    #[must_use]
    pub fn statement<'policy>(&self, policy: &'policy Policy) -> Statement<'policy> {
        let budgets = policy.budgets();
        Statement::new(
            budgets
                .0
                .iter()
                .map(|(id, budget)| {
                    let lines = budgets
                        .amount(budget)
                        .and_then(|allowance| budget.lines(&allowance));
                    Row::new(id, budget, lines, self.spend_of(budget))
                        .within(self.exposure_of(budget))
                })
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use meta_schema::Record;
    use meta_state::State;
    use settings::EmbeddedDefaults;

    use super::Ledger;
    use crate::budget::{Interval, Millis};
    use crate::budgets::{Amount, Budgets, ChangeLink, Measure, Standing};
    use crate::percent::Percent;
    use crate::policy::{Most, Policy};
    use crate::selection::Selection;

    const LOG: &str = include_str!("ledger/standing.jsonl");
    const STANDING: &str = include_str!("ledger/standing.toml");
    const TESTED: &str = "t-";
    const LINKED: &str = "7";
    const ADDRESSING: &str = "8";
    const BRANCH: &str = "task-0000000000000004";
    const STANDING_ROWS: [&str; 8] = [
        "t-concerns         features-and-maintenance   open_items concerns                  1          3          5 ok",
        "t-cost-concerns    performance-and-cost       open_items concerns                  1          0          0 over, outside the linked task's division",
        "t-cost-tasks       performance-and-cost       open_items tasks                     0          5          9 ok",
        "t-discussion       features-and-maintenance   open_items in_discussion             1          1          2 ok",
        "t-nav-issues       features-and-maintenance/nav open_items issues                    1          0          0 over, paid down by the linked task",
        "t-nav-tasks        features-and-maintenance/nav open_items tasks                     3          0          1 over",
        "t-nav-unheld       features-and-maintenance/nav open_items unheld_tasks              2          1          3 warn",
        "t-open-tasks       features-and-maintenance   open_items tasks                     4          2          4 warn",
    ];
    const BLOCKING: [&[&str]; 4] = [
        &["t-cost-concerns", "t-nav-issues", "t-nav-tasks"],
        &["t-nav-issues", "t-nav-tasks"],
        &["t-cost-concerns"],
        &["t-nav-tasks"],
    ];

    const NESTED: &str = "\n[domains.\"performance-and-cost/unit\"]\ntests = [\"*\"]\n\n[budgets.unit-time]\ndivision = \"performance-and-cost/unit\"\nmeasure = \"test_wall_time\"\nallowance = { of = \"test-time\", percent = 50 }\nwindow = \"per_change\"\nsource = \"ci\"\n";
    const STATEMENT: &str = include_str!("ledger/statement.txt");
    const MILLI: Millis = Millis::new(1);
    const BASELINED_SHARE: usize = 25;
    const BASELINED_TANGLE: usize = 64;

    impl Ledger {
        fn nested() -> Policy {
            toml::from_str(&format!("{}{NESTED}", Policy::DEFAULTS)).unwrap()
        }

        fn overspent(policy: &Policy) -> Ledger {
            let mut ledger = Ledger::default();
            let warned = policy
                .interval(Interval::PerTest)
                .checked_minus(MILLI)
                .unwrap_or_default();
            ledger.spend(
                Measure::TestWallTime,
                Amount::Duration(policy.interval(Interval::Total).plus(MILLI)),
            );
            ledger.spend(Measure::SlowestTest, Amount::Duration(warned));
            ledger.spend(
                Measure::TestLineShare,
                Amount::Share(Percent::from(BASELINED_SHARE)),
            );
            ledger.spend(
                Measure::TangleItems,
                Amount::Count(Most::from(BASELINED_TANGLE * 2)),
            );
            ledger
        }
    }

    impl Ledger {
        fn standing(link: &ChangeLink) -> Ledger {
            let records: Vec<Record> = LOG
                .lines()
                .map(|line| serde_json::from_str(line).unwrap())
                .collect();
            let mut ledger = Ledger::default();
            ledger.count(Some(Standing::read(&State::fold(&records), link)));
            ledger
        }

        fn blocking(&self, policy: &Policy) -> Vec<String> {
            let unreached = Selection::Crates(BTreeSet::new());
            self.statement(policy)
                .verdict(&unreached)
                .violations()
                .iter()
                .filter_map(|violation| {
                    violation
                        .message()
                        .split_whitespace()
                        .next()
                        .filter(|id| id.starts_with(TESTED))
                        .map(str::to_owned)
                })
                .collect()
        }
    }

    #[test]
    fn a_standing_budget_counts_the_open_items_of_its_division_subtree() {
        let policy: Policy = toml::from_str(&format!("{}{STANDING}", Policy::DEFAULTS)).unwrap();
        let ledger = Ledger::standing(&ChangeLink::new(Some(ADDRESSING.parse().unwrap()), None));
        let shown = ledger.statement(&policy).to_string();
        let table: Vec<&str> = shown
            .lines()
            .filter(|line| line.starts_with(TESTED))
            .collect();
        assert_eq!(table, STANDING_ROWS);
        let standing = [policy.budgets().standing(), Budgets::default().standing()];
        assert_eq!(standing, [true, false]);
    }

    #[test]
    fn an_over_standing_budget_blocks_a_change_linked_inside_its_division_unless_it_pays_down() {
        let policy: Policy = toml::from_str(&format!("{}{STANDING}", Policy::DEFAULTS)).unwrap();
        let links = [
            ChangeLink::default(),
            ChangeLink::new(Some(LINKED.parse().unwrap()), None),
            ChangeLink::new(None, Some(BRANCH.parse().unwrap())),
            ChangeLink::new(Some(ADDRESSING.parse().unwrap()), None),
        ];
        let blocking = links.map(|link| Ledger::standing(&link).blocking(&policy));
        let expected = BLOCKING.map(|ids| ids.iter().map(|id| (*id).to_owned()).collect());
        let unread = Ledger::default().blocking(&policy);
        assert_eq!((blocking, unread), (expected, Vec::new()));
    }

    #[test]
    fn an_overspend_fails_a_change_within_reach_and_a_nested_part_is_measured() {
        let policy = Ledger::nested();
        let ledger = Ledger::overspent(&policy);
        assert_eq!(ledger.statement(&policy).to_string(), STATEMENT);
        let unreached = Selection::Crates(BTreeSet::new());
        assert!(
            ledger.checked(&policy, &Selection::Workspace).is_err()
                && ledger.checked(&policy, &unreached).is_ok()
        );
    }
}
