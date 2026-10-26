use std::path::Path;

use serde::Deserialize;

use crate::rule::crate_graph::conductance::Conductance;
use crate::rule::crate_graph::crate_summary::CrateSummary;
use crate::rule::crate_graph::item_count::ItemCount;
use crate::site::Site;
use crate::violation::Violation;
use crate::violation_count::ViolationCount;

#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphLimits {
    max_scc: Option<ItemCount>,
    min_conductance: Option<Conductance>,
    min_side: Option<ItemCount>,
}

impl GraphLimits {
    pub fn violations(self, root: &Path, summary: &CrateSummary) -> Vec<Violation> {
        let tangles = self.max_scc.into_iter().flat_map(|max| {
            summary.tangles().iter().filter(move |tangle| tangle.items() > max).map(move |tangle| {
                Violation::weighted(Site::of_path(root), tangle, ViolationCount::of_len(tangle.items().excess_len(max)))
            })
        });
        let cut = summary
            .cut()
            .filter(|cut| self.min_conductance.is_some_and(|min| cut.conductance() < min))
            .filter(|cut| cut.smaller_side() >= self.min_side.unwrap_or(ItemCount::ONE))
            .map(|cut| Violation::new(Site::of_path(root), cut));
        tangles.chain(cut).collect()
    }
}
