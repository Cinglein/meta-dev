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
        self.mode = Mode::Enforcing;
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
                    (Exposure::Within, None) => {
                        Exposure::trend(self.based.get(measure), self.spent.get(measure))
                    }
                    (exposure, _) => exposure,
                }
            }
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

    use meta_golden::Golden;
    use meta_schema::Record;
    use meta_state::State;

    use super::{Ledger, Mode};
    use crate::budget::{Interval, Millis};
    use crate::budgets::{Amount, Budgets, ChangeLink, Measure, Standing};
    use crate::percent::Percent;
    use crate::policy::{Most, Policy};
    use crate::selection::Selection;

    const LOG: &str = include_str!("ledger/standing.jsonl");
    const STANDING: &str = include_str!("ledger/standing.toml");
    const BASE: &str = include_str!("ledger/base.toml");
    const ADDRESSING: &str = "8";
    const LINKS: [&str; 7] = [
        "", "7", "task-0000000000000004", ADDRESSING, "9", "10", "11",
    ];
    const NESTED: &str = "\n[domains.\"performance-and-cost/unit\"]\ntests = [\"*\"]\n\n[budgets.unit-time]\ndivision = \"performance-and-cost/unit\"\nmeasure = \"test_wall_time\"\nallowance = { of = \"test-time\", percent = 50 }\nwindow = \"per_change\"\nsource = \"ci\"\n";
    const MILLI: Millis = Millis::new(1);
    const BASELINED_SHARE: usize = 25;
    const TANGLED: Option<usize> = Some(64 * 2);
    const CAPPED: &str = concat!(
        include_str!("ledger/standing.toml"),
        "\n[budgets.t-tangle]\ndivision = \"features-and-maintenance\"\nmeasure = \"tangle_items\"\nallowance = { count = 4 }\nwindow = \"standing\"\n",
    );
    const OVER_CAP: Option<usize> = Some(6);
    const ENDS: [(&str, Option<usize>, Option<usize>); 8] = [
        ("7", Some(6), Some(5)),
        ("7", Some(5), Some(4)),
        ("7", Some(5), Some(5)),
        ("7", Some(5), Some(6)),
        ("7", None, Some(5)),
        ("7", Some(5), None),
        (ADDRESSING, Some(5), Some(6)),
        ("task-0000000000000004", Some(5), Some(6)),
    ];

    impl Ledger {
        fn shown(&self, policy: &Policy) -> String {
            let statement = self.statement(policy);
            let [unreached, reached] = [Selection::Crates(BTreeSet::new()), Selection::Workspace]
                .map(|selection| statement.verdict(&selection));
            format!("{statement}unreached {unreached}reached {reached}")
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
            ledger.tangled(TANGLED)
        }

        fn tangled(mut self, tangle: Option<usize>) -> Ledger {
            self.spent.extend(
                tangle.map(|tangle| (Measure::TangleItems, Amount::Count(Most::from(tangle)))),
            );
            self
        }

        fn standing(link: &str, policy: &Policy) -> Ledger {
            let records: Vec<Record> = LOG
                .lines()
                .map(|line| serde_json::from_str(line).unwrap())
                .collect();
            let link = ChangeLink::new(link.parse().ok(), link.parse().ok());
            let mut ledger = Ledger {
                mode: Mode::Enforcing,
                ..Ledger::default()
            };
            ledger.count(Some(Standing::read(
                &State::fold(&records),
                &link,
                policy.scales(),
            )));
            ledger
        }
    }

    #[test]
    fn an_over_standing_budget_blocks_a_change_linked_inside_its_division_unless_it_pays_down() {
        let policy = Policy::with_added(STANDING).unwrap();
        let shown = LINKS.map(|link| {
            format!(
                "link {link:?}\n{}",
                Ledger::standing(link, &policy).shown(&policy)
            )
        });
        let unlinked = Ledger::default().shown(&policy);
        Golden::for_test(module_path!(), "standing")
            .checked(&format!("{}{unlinked}", shown.concat()))
            .unwrap();
        assert!(policy.budgets().standing() && !Budgets::default().standing());
    }

    #[test]
    fn an_over_standing_measure_blocks_its_division_until_a_change_moves_it_toward_its_line() {
        let waiting = Policy::with_added(STANDING).unwrap();
        let mut ledger = Ledger::standing(ADDRESSING, &waiting);
        ledger.enforce_from(Some(BASE)).unwrap();
        let enforced = ledger.shown(&waiting);
        ledger.report_only();
        let reported = ledger.shown(&waiting);
        let policy = Policy::with_added(CAPPED).unwrap();
        let ends = ENDS.map(|(link, base, head)| {
            let mut ledger = Ledger::standing(link, &policy).tangled(head);
            ledger.enforce_from(Some(BASE)).unwrap();
            ledger.measured_at_base(Ledger::default().tangled(base));
            format!(
                "link {link:?} base {base:?} head {head:?}\n{}",
                ledger.shown(&policy)
            )
        });
        let fresh = Ledger::default().tangled(OVER_CAP).shown(&policy);
        Golden::for_test(module_path!(), "merge_base")
            .checked(&format!("{enforced}{reported}{}{fresh}", ends.concat()))
            .unwrap();
    }

    #[test]
    fn an_overspend_fails_a_change_within_reach_and_a_nested_part_is_measured() {
        let policy = Policy::with_added(NESTED).unwrap();
        let ledger = Ledger::overspent(&policy);
        let linted = Ledger::default().tangled(TANGLED);
        let shown = format!("{}{}", ledger.shown(&policy), linted.shown(&policy));
        Golden::for_test(module_path!(), "overspent")
            .checked(&shown)
            .unwrap();
        assert!(ledger.checked(&policy, &Selection::Workspace).is_err());
    }
}
