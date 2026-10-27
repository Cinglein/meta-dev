use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::process_row::GroupId;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MarkedGroups(BTreeSet<GroupId>);

impl MarkedGroups {
    pub fn holds(&self, group: GroupId) -> Option<()> {
        self.0.contains(&group).then_some(())
    }

    #[must_use]
    pub fn renewed(&self, live: &BTreeSet<GroupId>, marked: BTreeSet<GroupId>) -> Self {
        Self(self.0.intersection(live).copied().chain(marked).collect())
    }
}
