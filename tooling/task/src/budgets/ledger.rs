mod mode;
mod prior_standing;
mod row;
mod statement;

use std::collections::BTreeMap;

use super::budget::{Amount, Budget, Measure};
use super::budget_id::BudgetId;
use super::standing::{Exposure, Standing};
use crate::failure::Failure;
use crate::policy::Policy;
use crate::selection::Selection;
use mode::Mode;
use prior_standing::PriorStanding;
use row::Row;
use statement::Statement;

#[derive(Debug, Default)]
pub struct Ledger {
    spent: BTreeMap<Measure, Amount>,
    based: BTreeMap<Measure, Amount>,
    standing: Option<Standing>,
    prior: Option<PriorStanding>,
    mode: Mode,
}

impl Ledger {
    pub fn report_only(&mut self) {
        self.mode = Mode::Reporting;
    }

    #[must_use]
    pub fn reports_only(&self) -> bool {
        self.mode == Mode::Reporting
    }

    pub fn enforce_from(&mut self, policy_text: Option<&str>) -> Result<(), toml::de::Error> {
        self.prior = Some(policy_text.map_or_else(|| Ok(PriorStanding::default()), str::parse)?);
        Ok(())
    }

    #[must_use]
    pub fn reads_standing(&self, policy: &Policy) -> bool {
        policy.budgets().standing()
            || self
                .prior
                .as_ref()
                .is_some_and(|prior| prior.budgets().standing())
    }

    pub fn spend(&mut self, measure: Measure, amount: Amount) {
        self.spent.insert(measure, amount);
    }

    pub fn measured_at_base(&mut self, base: Ledger) {
        self.based = base.spent;
    }

    pub fn count(&mut self, standing: Option<Standing>) {
        self.standing = standing;
    }

    #[must_use]
    pub fn spend_of(&self, budget: &Budget) -> Option<Amount> {
        budget.measure().item_filter().map_or_else(
            || self.spent.get(budget.measure()).cloned(),
            |filter| {
                self.standing
                    .as_ref()
                    .map(|standing| standing.spend(filter, budget.division()))
            },
        )
    }

    #[must_use]
    pub fn exposure_of(&self, budget: &Budget) -> Exposure {
        let measure = budget.measure();
        match (budget.standing_in(), self.mode, self.standing.as_ref()) {
            (None, ..) => Exposure::Diff,
            (Some(_), Mode::Reporting, _) => Exposure::Reported,
            (Some(_), Mode::Enforcing, None) => Exposure::Within,
            (Some(division), Mode::Enforcing, Some(standing)) => {
                let ranked = measure
                    .item_filter()
                    .cloned()
                    .or_else(|| standing.least_severe(division));
                match (
                    standing.exposure(ranked.as_ref(), division),
                    measure.item_filter(),
                ) {
                    (Exposure::Within, None) => self.trend(measure),
                    (exposure, _) => exposure,
                }
            }
        }
    }

    fn trend(&self, measure: &Measure) -> Exposure {
        match (self.based.get(measure), self.spent.get(measure)) {
            (Some(base), Some(head)) if head < base => Exposure::Improves,
            (Some(base), Some(head)) if head == base => Exposure::Unchanged,
            (Some(_), Some(_)) => Exposure::Within,
            (None, _) | (_, None) => Exposure::Unmeasured,
        }
    }

    pub fn checked(&self, policy: &Policy, selection: &Selection) -> Result<(), Failure> {
        let statement = self.statement(policy);
        print!("{statement}");
        statement.verdict(selection).verdict()
    }

    #[must_use]
    pub fn statement<'view>(&'view self, policy: &'view Policy) -> Statement<'view> {
        let current = policy.budgets();
        let prior = self.prior.as_ref().map(PriorStanding::budgets);
        let waiting = |id: &BudgetId, budget: &Budget| {
            prior.is_some_and(|prior| budget.standing_in().is_some() && !prior.0.contains_key(id))
        };
        let enforced = |id: &BudgetId, budget: &Budget| {
            prior.is_none_or(|_| budget.standing_in().is_none() || waiting(id, budget))
        };
        let own = current.0.iter().filter(|(id, budget)| enforced(id, budget));
        let held = prior.into_iter().flat_map(|prior| prior.0.iter());
        let rows: BTreeMap<&BudgetId, Row> = own
            .map(|(id, budget)| (current, id, budget))
            .chain(held.map(|(id, budget)| (prior.unwrap_or(current), id, budget)))
            .map(|(budgets, id, budget)| {
                let lines = budgets
                    .amount(budget)
                    .and_then(|allowance| budget.lines(&allowance));
                let exposure = if waiting(id, budget) {
                    Exposure::Pending
                } else {
                    self.exposure_of(budget)
                };
                (
                    id,
                    Row::new(id, budget, lines, self.spend_of(budget)).within(exposure),
                )
            })
            .collect();
        Statement::new(rows.into_values().collect())
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
    pub(super) const STANDING: &str = include_str!("ledger/standing.toml");
    const TESTED: &str = "t-";
    const CRITICAL: &str = "open_items critical";
    pub(super) const ADDRESSING: &str = "8";
    const STANDING_ROWS: [&str; 13] = [
        "architecture-and-planning-critical architecture-and-planning  open_items critical                  0          0          0 ok",
        "features-and-maintenance-critical features-and-maintenance   open_items critical                  1          0          0 over, paid down by the linked task",
        "performance-and-cost-critical performance-and-cost       open_items critical                  1          0          0 over, outside the linked task's division",
        "t-concerns         features-and-maintenance   open_items concerns                  1          3          5 ok",
        "t-cost-concerns    performance-and-cost       open_items concerns                  1          0          0 over, outside the linked task's division",
        "t-cost-tasks       performance-and-cost       open_items tasks                     0          5          9 ok",
        "t-discussion       features-and-maintenance   open_items in_discussion             1          1          2 ok",
        "t-nav-concerns     features-and-maintenance/nav open_items concerns                  1          0          0 over, paid down by the linked task",
        "t-nav-tasks        features-and-maintenance/nav open_items tasks                     4          0          1 over",
        "t-nav-unheld       features-and-maintenance/nav open_items unheld_tasks              3          1          3 warn",
        "t-open-tasks       features-and-maintenance   open_items tasks                     5          3          6 warn",
        "t-planning-issues  architecture-and-planning  open_items issues                    1          0          0 over, outside the linked task's division",
        "t-unheld           features-and-maintenance   open_items unheld_tasks              3          3          5 ok",
    ];
    const ALL_OVER: &[&str] = &[
        "t-cost-concerns", "t-nav-concerns", "t-nav-tasks", "t-planning-issues",
    ];
    const UNREACHED: [(&str, &[&str]); 7] = [
        ("", ALL_OVER),
        ("7", &["t-nav-concerns", "t-nav-tasks"]),
        ("task-0000000000000004", &["t-cost-concerns"]),
        (ADDRESSING, &["t-nav-tasks"]),
        ("9", ALL_OVER),
        ("10", &["t-nav-concerns", "t-nav-tasks"]),
        ("11", &["t-nav-concerns", "t-nav-tasks"]),
    ];
    const DIFF_REACHED: [(&str, &[&str]); 3] = [
        ("task-0000000000000004", &["t-cost-concerns"]),
        ("7", &["t-nav-concerns", "t-nav-tasks"]),
        (ADDRESSING, &["t-nav-tasks"]),
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
            ledger.tangled();
            ledger
        }

        fn tangled(&mut self) {
            self.spend(
                Measure::TangleItems,
                Amount::Count(Most::from(BASELINED_TANGLE * 2)),
            );
        }
    }

    impl Ledger {
        pub(super) fn standing(link: &str, policy: &Policy) -> Ledger {
            let records: Vec<Record> = LOG
                .lines()
                .map(|line| serde_json::from_str(line).unwrap())
                .collect();
            let link = ChangeLink::new(link.parse().ok(), link.parse().ok());
            let mut ledger = Ledger::default();
            ledger.count(Some(Standing::read(
                &State::fold(&records),
                &link,
                policy.scales(),
            )));
            ledger
        }

        pub(super) fn blocking(&self, policy: &Policy, selection: &Selection) -> Vec<String> {
            self.statement(policy)
                .verdict(selection)
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
        let shown = Ledger::standing(ADDRESSING, &policy)
            .statement(&policy)
            .to_string();
        let table: Vec<&str> = shown
            .lines()
            .filter(|line| line.starts_with(TESTED) || line.contains(CRITICAL))
            .collect();
        assert_eq!(table, STANDING_ROWS);
        let standing = [policy.budgets().standing(), Budgets::default().standing()];
        assert_eq!(standing, [true, false]);
    }

    #[test]
    fn an_over_standing_budget_blocks_a_change_linked_inside_its_division_unless_it_pays_down() {
        let policy: Policy = toml::from_str(&format!("{}{STANDING}", Policy::DEFAULTS)).unwrap();
        let unreached = Selection::Crates(BTreeSet::new());
        let blocking = |selection: &Selection, (link, _): (&str, &[&str])| {
            Ledger::standing(link, &policy).blocking(&policy, selection)
        };
        let expected = |(_, ids): (&str, &[&str])| -> Vec<String> {
            ids.iter().map(|id| (*id).to_owned()).collect()
        };
        assert_eq!(
            UNREACHED.map(|case| blocking(&unreached, case)),
            UNREACHED.map(expected)
        );
        assert_eq!(
            DIFF_REACHED.map(|case| blocking(&Selection::Workspace, case)),
            DIFF_REACHED.map(expected)
        );
        assert_eq!(
            Ledger::default().blocking(&policy, &unreached),
            Vec::<String>::new()
        );
    }

    #[test]
    fn an_overspend_fails_a_change_within_reach_and_a_nested_part_is_measured() {
        let policy = Ledger::nested();
        let ledger = Ledger::overspent(&policy);
        assert_eq!(ledger.statement(&policy).to_string(), STATEMENT);
        let unreached = Selection::Crates(BTreeSet::new());
        let mut linted = Ledger::default();
        linted.tangled();
        assert!(
            ledger.checked(&policy, &Selection::Workspace).is_err()
                && ledger.checked(&policy, &unreached).is_ok()
                && linted.checked(&policy, &Selection::Workspace).is_ok()
        );
    }
}
