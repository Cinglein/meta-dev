mod row;
mod statement;

use std::collections::BTreeMap;

use super::budget::{Amount, Budget, Measure};
use crate::domains::{Domains, Extent};
use crate::failure::Failure;
use crate::policy::Policy;
use crate::selection::Selection;
use row::Row;
use statement::Statement;

#[derive(Debug, Default)]
pub struct Ledger {
    spent: BTreeMap<Measure, Amount>,
    ratchets: BTreeMap<Measure, Amount>,
}

impl Ledger {
    pub fn spend(&mut self, measure: Measure, amount: Amount) {
        self.spent.insert(measure, amount);
    }

    pub fn ratchet(&mut self, measure: Measure, allowed: Amount, amount: Amount) {
        self.ratchets.insert(measure, allowed);
        self.spend(measure, amount);
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
                    let allowance = budgets
                        .amount(budget)
                        .filter(|amount| amount.baseline().is_none())
                        .or_else(|| self.ratchets.get(&budget.measure()).cloned());
                    let lines = allowance.and_then(|allowance| budget.lines(&allowance));
                    Row::new(id, budget, lines, self.spent_on(budget, policy.domains()))
                })
                .collect(),
        )
    }

    fn spent_on(&self, budget: &Budget, domains: &Domains) -> Option<Amount> {
        domains
            .domain(budget.division())
            .filter(|domain| domain.extent(budget.measure().scope()) == Extent::Whole)
            .and(self.spent.get(&budget.measure()))
            .cloned()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use settings::EmbeddedDefaults;

    use super::Ledger;
    use crate::budget::{Interval, Millis};
    use crate::budgets::{Amount, Measure};
    use crate::policy::Policy;
    use crate::selection::Selection;

    const NESTED: &str = "\n[domains.\"performance-and-cost/unit\"]\ntests = [\"meta_*\"]\n\n[budgets.unit-time]\ndivision = \"performance-and-cost/unit\"\nmeasure = \"test_wall_time\"\nallowance = { of = \"test-time\", percent = 50 }\nwindow = \"per_change\"\nsource = \"local_gate\"\n";
    const STATEMENT: &str = include_str!("ledger/statement.txt");
    const MILLI: Millis = Millis::new(1);
    const WHOLE_TESTS: &str = "tests = [\"*\"]";
    const NARROW_TESTS: &str = "tests = [\"meta_*\"]";

    impl Ledger {
        fn nested() -> Policy {
            toml::from_str(&format!("{}{NESTED}", Policy::DEFAULTS)).unwrap()
        }

        fn narrowed() -> Policy {
            toml::from_str(&Policy::DEFAULTS.replace(WHOLE_TESTS, NARROW_TESTS)).unwrap()
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
            ledger
        }
    }

    #[test]
    fn an_overspend_fails_a_change_within_reach_and_a_partial_domain_is_unmeasured() {
        let policy = Ledger::nested();
        let ledger = Ledger::overspent(&policy);
        assert_eq!(ledger.statement(&policy).to_string(), STATEMENT);
        let unreached = Selection::Crates(BTreeSet::new());
        assert!(
            ledger.checked(&policy, &Selection::Workspace).is_err()
                && ledger.checked(&policy, &unreached).is_ok()
        );
    }

    #[test]
    fn a_top_division_with_a_partial_test_domain_leaves_test_time_unmeasured() {
        let policy = Ledger::narrowed();
        let ledger = Ledger::overspent(&policy);
        assert!(ledger.checked(&policy, &Selection::Workspace).is_ok());
    }
}
