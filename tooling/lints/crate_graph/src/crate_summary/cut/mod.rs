mod condensed;
mod conductance;

use std::collections::BTreeSet;
use std::fmt;

use crate::crate_summary::cut::conductance::Conductance;
use crate::item_count::ItemCount;
use crate::item_graph::{ItemGraph, NodeIndex};
use condensed::{Condensed, TangleIndex};
use task::Name;

#[derive(Clone, Debug, PartialEq)]
pub struct Cut {
    conductance: Conductance,
    inside: Vec<Name>,
    outside: Vec<Name>,
}

impl Cut {
    pub fn best(graph: &ItemGraph, tangles: &[Vec<NodeIndex>]) -> Option<Cut> {
        let condensed = Condensed::of_tangles(graph, tangles);
        Self::best_side(&condensed).map(|side| Cut {
            conductance: condensed.conductance(&side),
            inside: Self::names(graph, tangles, &side),
            outside: Self::names(
                graph,
                tangles,
                &condensed.all().difference(&side).copied().collect(),
            ),
        })
    }

    pub fn conductance(&self) -> Conductance {
        self.conductance
    }

    pub fn smaller_side(&self) -> ItemCount {
        ItemCount::of_len(self.inside.len().min(self.outside.len()))
    }

    fn best_side(condensed: &Condensed) -> Option<BTreeSet<TangleIndex>> {
        let islands = condensed.islands();
        if islands.len() > 1 {
            Some(Self::balanced(condensed, islands))
        } else {
            let order = condensed.sweep_order();
            (1..order.len())
                .map(|prefix| {
                    order
                        .iter()
                        .take(prefix)
                        .copied()
                        .collect::<BTreeSet<TangleIndex>>()
                })
                .min_by(|left, right| {
                    condensed
                        .conductance(left)
                        .total_cmp(condensed.conductance(right))
                })
        }
    }

    fn balanced(
        condensed: &Condensed,
        islands: Vec<BTreeSet<TangleIndex>>,
    ) -> BTreeSet<TangleIndex> {
        let mut islands = islands;
        islands.sort_by_key(|island| std::cmp::Reverse(condensed.items(island)));
        let total = condensed.items(&condensed.all());
        islands.into_iter().fold(BTreeSet::new(), |inside, island| {
            if condensed.items(&inside) + condensed.items(&inside) <= total {
                inside.into_iter().chain(island).collect()
            } else {
                inside
            }
        })
    }

    fn names(
        graph: &ItemGraph,
        tangles: &[Vec<NodeIndex>],
        side: &BTreeSet<TangleIndex>,
    ) -> Vec<Name> {
        side.iter()
            .filter_map(|tangle| tangles.get(tangle.index()))
            .flatten()
            .filter_map(|member| graph.name_at(*member))
            .cloned()
            .collect()
    }
}

impl fmt::Display for Cut {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let list = |names: &[Name]| {
            names
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<String>>()
                .join(" ")
        };
        write!(
            formatter,
            "conductance {} between {} items [{}] and {} items [{}]",
            self.conductance,
            self.inside.len(),
            list(&self.inside),
            self.outside.len(),
            list(&self.outside)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::Cut;
    use crate::crate_summary::cut::conductance::Conductance;
    use crate::item_count::ItemCount;
    use crate::item_graph::ItemGraph;
    use task::SourceFile;

    const CLUSTERS: &str = "struct A(B, C); struct B(A, C); struct C(A, B, D); struct D(E, F); struct E(D, F); struct F(D, E);";
    const TIGHT: &str = "struct A(B, C, D); struct B(C, D); struct C(D); struct D;";
    const PATH: &str = "crates/a/src/lib.rs";
    const SIDE_LEN: usize = 3;

    #[test]
    fn two_weakly_linked_clusters_split_between_them() {
        let file = SourceFile::parse(PATH, CLUSTERS.to_owned()).expect("valid rust");
        let graph = ItemGraph::of_files(&[&file]);
        let cut = Cut::best(&graph, &graph.tangles()).expect("a cut");
        assert_eq!(cut.smaller_side(), ItemCount::of_len(SIDE_LEN));
        assert!(cut.conductance() < Conductance::of_ratio(1.0, 10.0));
    }

    #[test]
    fn a_tight_cluster_has_no_cheap_cut() {
        let file = SourceFile::parse(PATH, TIGHT.to_owned()).expect("valid rust");
        let graph = ItemGraph::of_files(&[&file]);
        let cut = Cut::best(&graph, &graph.tangles()).expect("a cut");
        assert!(cut.conductance() > Conductance::of_ratio(1.0, 2.0));
    }
}
