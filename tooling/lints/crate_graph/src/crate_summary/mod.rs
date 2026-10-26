mod cut;
mod tangle;

use std::path::PathBuf;

use task::{Floor, Limit, Policy, Share, Site, Violation};

use crate::item_graph::ItemGraph;
use cut::Cut;
use tangle::Tangle;

#[derive(Clone, Debug)]
pub struct CrateSummary {
    root: PathBuf,
    tangles: Vec<Tangle>,
    cut: Option<Cut>,
}

impl CrateSummary {
    pub fn of_graph(root: PathBuf, graph: &ItemGraph) -> CrateSummary {
        let members = graph.tangles();
        let tangles = members
            .iter()
            .map(|members| {
                Tangle::new(
                    members
                        .iter()
                        .filter_map(|member| graph.name_at(*member))
                        .cloned()
                        .collect(),
                )
            })
            .collect();
        CrateSummary {
            root,
            tangles,
            cut: Cut::best(graph, &members),
        }
    }

    pub fn violations(&self, policy: &Policy) -> Vec<Violation> {
        let site = || Site::File(self.root.clone());
        let tangles = self
            .tangles
            .iter()
            .filter(|tangle| !policy.limit(Limit::SccItems).allows(tangle.items().count()))
            .map(|tangle| Violation::new(site(), tangle));
        let cut = self
            .cut
            .iter()
            .filter(|cut| {
                policy
                    .share(Share::CutConductance)
                    .exceeds(cut.conductance().ratio())
            })
            .filter(|cut| {
                policy
                    .floor(Floor::CutSide)
                    .allows(cut.smaller_side().count())
            })
            .map(|cut| Violation::new(site(), cut));
        tangles.chain(cut).collect()
    }
}
