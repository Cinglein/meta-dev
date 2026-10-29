use meta_schema::{Division, NewTask, Placement, Target};
use meta_severity::Scales;
use meta_state::RatedItem;

use crate::budgets::budget::ItemFilter;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenItem {
    target: Option<Target>,
    placement: Placement,
    marks: Vec<ItemFilter>,
    rated: Option<RatedItem>,
}

impl OpenItem {
    #[must_use]
    pub fn new(
        target: Option<Target>,
        placement: Placement,
        marks: Vec<ItemFilter>,
        rated: Option<RatedItem>,
    ) -> OpenItem {
        OpenItem {
            target,
            placement,
            marks,
            rated,
        }
    }

    #[must_use]
    pub fn counts_in(&self, filter: &ItemFilter, division: &Division, scales: &Scales) -> bool {
        filter.severity().map_or_else(
            || {
                self.marks.contains(filter)
                    && self.placement.division().is_some_and(|placed| {
                        division == placed || division.strictly_contains(placed)
                    })
            },
            |severity| {
                self.rated
                    .as_ref()
                    .is_some_and(|rated| severity.admitted(scales, division, rated).is_some())
            },
        )
    }

    #[must_use]
    pub fn pays_into(
        &self,
        filter: &ItemFilter,
        division: &Division,
        scales: &Scales,
    ) -> Option<&OpenItem> {
        let covered = filter.severity().map_or_else(
            || self.counts_in(filter, division, scales),
            |severity| {
                self.rated
                    .as_ref()
                    .and_then(|rated| severity.covers(rated, scales, division))
                    .is_some()
            },
        );
        covered.then_some(self)
    }

    #[must_use]
    pub fn addressed_by(&self, task: &NewTask) -> bool {
        self.target
            .is_some_and(|target| task.addresses().contains(&target))
    }
}
