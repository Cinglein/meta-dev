use meta_schema::{Division, NewTask, Placement, Target};

use crate::budgets::budget::ItemFilter;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenItem {
    target: Option<Target>,
    placement: Placement,
    marks: Vec<ItemFilter>,
}

impl OpenItem {
    #[must_use]
    pub fn new(target: Option<Target>, placement: Placement, marks: Vec<ItemFilter>) -> OpenItem {
        OpenItem {
            target,
            placement,
            marks,
        }
    }

    #[must_use]
    pub fn counts_in(&self, filter: ItemFilter, division: &Division) -> bool {
        self.marks.contains(&filter)
            && self
                .placement
                .division()
                .is_some_and(|placed| division == placed || division.strictly_contains(placed))
    }

    #[must_use]
    pub fn addressed_by(&self, task: &NewTask) -> bool {
        self.target
            .is_some_and(|target| task.addresses().contains(&target))
    }
}
