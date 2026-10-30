use std::collections::BTreeMap;

use meta_amount::Percent;
use meta_schema::BudgetId;
use serde::{Deserialize, Serialize};

use super::Budgets;
use super::budget::{Amount, Budget, Measure};
use super::legacy_key::LegacyKey;
use super::tree_failure::TreeFailure;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BudgetDrafts(BTreeMap<BudgetId, Budget>);

impl BudgetDrafts {
    #[must_use]
    pub fn measures(&self, measure: &Measure) -> bool {
        self.0
            .values()
            .any(|budget| budget.top_measure().as_ref() == Some(measure))
    }

    pub fn adding(
        mut self,
        key: LegacyKey,
        amount: Amount,
        tolerance: Option<Percent>,
    ) -> Result<BudgetDrafts, TreeFailure> {
        match self.0.insert(
            key.budget_id(),
            Budget::legacy(key.measure(), amount, tolerance),
        ) {
            Some(_) => Err(TreeFailure::Taken(key.budget_id())),
            None => Ok(self),
        }
    }
}

impl From<BudgetDrafts> for BTreeMap<BudgetId, Budget> {
    fn from(drafts: BudgetDrafts) -> BTreeMap<BudgetId, Budget> {
        drafts.0
    }
}

impl From<Budgets> for BudgetDrafts {
    fn from(budgets: Budgets) -> BudgetDrafts {
        BudgetDrafts(budgets.0)
    }
}
