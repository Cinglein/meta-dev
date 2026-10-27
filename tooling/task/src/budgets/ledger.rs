mod contents;
mod mode;
mod prior_standing;
mod row;
mod statement;

use std::collections::BTreeMap;
use std::path::PathBuf;

use super::Budgets;
use super::budget::{Amount, Budget, Measure, Tally};
use super::budget_id::BudgetId;
use super::standing::{Exposure, Standing};
use crate::domains::Domains;
use crate::failure::Failure;
use crate::policy::{Policy, SpecSettings};
use crate::selection::Selection;
use contents::Contents;
use mode::Mode;
use prior_standing::PriorStanding;
use row::Row;
use statement::Statement;

#[derive(Debug, Default)]
pub struct Ledger {
    spent: BTreeMap<Measure, Amount>,
    standing: Option<Standing>,
    contents: Option<Contents>,
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

    pub fn count(&mut self, standing: Option<Standing>) {
        self.standing = standing;
    }

    pub fn measure(&mut self, path: PathBuf, text: &str, spec: Option<&SpecSettings>) {
        self.contents.get_or_insert_default().hold(path, text, spec);
    }

    #[must_use]
    pub fn spend_of(&self, budget: &Budget, domains: &Domains) -> Option<Amount> {
        let measure = budget.measure();
        match (measure.item_filter(), measure.tally()) {
            (Some(filter), _) => self
                .standing
                .as_ref()
                .map(|standing| standing.spend(filter, budget.division())),
            (None, Tally::DomainFiles) => self
                .contents
                .as_ref()
                .zip(domains.domain(budget.division()))
                .map(|(contents, domain)| contents.words(domain)),
            (None, Tally::WholeScope) => self.spent.get(measure).cloned(),
        }
    }

    #[must_use]
    pub fn exposure_of(&self, budget: &Budget, domains: &Domains) -> Exposure {
        budget.standing_in().zip(self.standing.as_ref()).map_or(
            Exposure::Diff,
            |(division, standing)| match self.mode {
                Mode::Reporting => Exposure::Reported,
                Mode::Enforcing => {
                    standing.exposure(budget.measure().item_filter(), division, domains)
                }
            },
        )
    }

    pub fn checked(&self, policy: &Policy, selection: &Selection) -> Result<(), Failure> {
        let statement = self.statement(policy);
        print!("{statement}");
        statement.verdict(selection).verdict()
    }

    #[must_use]
    pub fn statement<'view>(&'view self, policy: &'view Policy) -> Statement<'view> {
        let prior = self.prior.as_ref();
        let waiting = move |id: &BudgetId, budget: &Budget| {
            prior.is_some_and(|prior| {
                budget.standing_in().is_some() && !prior.budgets().0.contains_key(id)
            })
        };
        let enforced = |id: &BudgetId, budget: &Budget| {
            prior.is_none_or(|_| budget.standing_in().is_none() || waiting(id, budget))
        };
        let row = move |budgets: &'view Budgets,
                        domains: &'view Domains,
                        id: &'view BudgetId,
                        budget: &'view Budget| {
            let lines = budgets
                .amount(budget)
                .and_then(|allowance| budget.lines(&allowance));
            let exposure = if waiting(id, budget) {
                Exposure::Pending
            } else {
                self.exposure_of(budget, domains)
            };
            let spend = self.spend_of(budget, domains);
            (id, Row::new(id, budget, lines, spend).within(exposure))
        };
        let own = policy
            .budgets()
            .0
            .iter()
            .filter(|(id, budget)| enforced(id, budget))
            .map(|(id, budget)| row(policy.budgets(), policy.domains(), id, budget));
        let held = prior.into_iter().flat_map(|prior| {
            prior
                .budgets()
                .0
                .iter()
                .map(move |(id, budget)| row(prior.budgets(), prior.domains(), id, budget))
        });
        let rows: BTreeMap<&BudgetId, Row> = own.chain(held).collect();
        Statement::new(rows.into_values().collect())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::path::PathBuf;

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
    const STANDING_ROWS: [&str; 15] = [
        "architecture-and-planning-critical architecture-and-planning  open_items critical                  0          0          0 ok",
        "features-and-maintenance-critical features-and-maintenance   open_items critical                  1          0          0 over, paid down by the linked task",
        "performance-and-cost-critical performance-and-cost       open_items critical                  1          0          0 over, outside the linked task's division; a diff in its domain reaches it",
        "t-concerns         features-and-maintenance   open_items concerns                  1          3          5 ok",
        "t-cost-concerns    performance-and-cost       open_items concerns                  1          0          0 over, outside the linked task's division; a diff in its domain reaches it",
        "t-cost-tasks       performance-and-cost       open_items tasks                     0          5          9 ok",
        "t-discussion       features-and-maintenance   open_items in_discussion             1          1          2 ok",
        "t-nav-concerns     features-and-maintenance/nav open_items concerns                  1          0          0 over, paid down by the linked task",
        "t-nav-tasks        features-and-maintenance/nav open_items tasks                     4          0          1 over",
        "t-nav-unheld       features-and-maintenance/nav open_items unheld_tasks              3          1          3 warn",
        "t-nav-words        features-and-maintenance/nav words                                3          1          2 over",
        "t-open-tasks       features-and-maintenance   open_items tasks                     5          3          6 warn",
        "t-planning-issues  architecture-and-planning  open_items issues                    1          0          0 over, outside the linked task's division",
        "t-unheld           features-and-maintenance   open_items unheld_tasks              3          3          5 ok",
        "t-verbose          features-and-maintenance   open_items verbose 3                 3          2          4 warn",
    ];
    const ALL_OVER: &[&str] = &[
        "t-cost-concerns", "t-nav-concerns", "t-nav-tasks", "t-nav-words", "t-planning-issues",
    ];
    const NAV_OVER: &[&str] = &["t-nav-concerns", "t-nav-tasks", "t-nav-words"];
    const UNREACHED: [(&str, &[&str]); 7] = [
        ("", ALL_OVER),
        ("7", NAV_OVER),
        ("task-0000000000000004", &["t-cost-concerns"]),
        (ADDRESSING, &["t-nav-tasks", "t-nav-words"]),
        ("9", ALL_OVER),
        ("10", NAV_OVER),
        ("11", NAV_OVER),
    ];
    const DIFF_REACHED: [(&str, &[&str]); 3] = [
        (
            "task-0000000000000004",
            &[
                "t-cost-concerns", "t-nav-concerns", "t-nav-tasks", "t-nav-words",
            ],
        ),
        (
            "7",
            &[
                "t-cost-concerns", "t-nav-concerns", "t-nav-tasks", "t-nav-words",
            ],
        ),
        (
            ADDRESSING,
            &["t-cost-concerns", "t-nav-tasks", "t-nav-words"],
        ),
    ];
    const PROSE: [(&str, &str); 2] = [("crates/a.md", "a b c"), ("README.md", "a b c d")];

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
            PROSE.iter().fold(ledger, |mut ledger, (path, text)| {
                ledger.measure(PathBuf::from(path), text, policy.spec());
                ledger
            })
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
        assert!(
            ledger.checked(&policy, &Selection::Workspace).is_err()
                && ledger.checked(&policy, &unreached).is_ok()
        );
    }
}
