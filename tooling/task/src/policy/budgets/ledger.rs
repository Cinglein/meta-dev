mod row;
mod statement;

use std::collections::BTreeMap;

use super::budget::{Amount, Measure};
use crate::failure::Failure;
use crate::policy::Policy;
use crate::policy::domains::Extent;
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
                    let spend = self.spent.get(&budget.measure()).filter(|_| {
                        policy
                            .domains()
                            .domain(budget.division())
                            .is_some_and(|domain| {
                                domain.extent(budget.measure().scope()) == Extent::Whole
                            })
                    });
                    Row::new(
                        id,
                        budget,
                        allowance.and_then(|allowance| budget.lines(&allowance)),
                        spend.cloned(),
                    )
                })
                .collect(),
        )
    }
}
