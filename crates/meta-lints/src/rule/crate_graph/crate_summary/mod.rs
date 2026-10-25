mod cut;
mod tangle;

use std::fmt;
use std::path::PathBuf;

use crate::rule::crate_graph::item_count::ItemCount;
use crate::rule::crate_graph::item_graph::ItemGraph;
use cut::Cut;
use tangle::Tangle;

#[derive(Clone, Debug)]
pub struct CrateSummary {
    root: PathBuf,
    items: ItemCount,
    edges: ItemCount,
    tangles: Vec<Tangle>,
    cut: Option<Cut>,
}

impl CrateSummary {
    pub fn of_graph(root: PathBuf, graph: &ItemGraph) -> CrateSummary {
        let members = graph.tangles();
        let tangles = members
            .iter()
            .filter(|members| members.len() > 1)
            .map(|members| Tangle::new(members.iter().filter_map(|member| graph.name_at(*member)).cloned().collect()))
            .collect();
        CrateSummary {
            root,
            items: graph.items(),
            edges: ItemCount::of_len(graph.edges().len()),
            tangles,
            cut: Cut::best(graph, &members),
        }
    }

    pub fn tangles(&self) -> &[Tangle] {
        &self.tangles
    }

    pub fn cut(&self) -> Option<&Cut> {
        self.cut.as_ref()
    }
}

impl fmt::Display for CrateSummary {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "{}: {} items, {} edges", self.root.display(), self.items, self.edges)?;
        self.tangles.iter().try_for_each(|tangle| writeln!(formatter, "  {tangle}"))?;
        self.cut.iter().try_for_each(|cut| writeln!(formatter, "  best cut: {cut}"))
    }
}
