mod concept;
mod concept_report;
mod distribution;
mod fact_source;
mod facts;
mod formal_context;
mod hundredths;
mod impl_fact;
mod impl_kind;
mod implementations;
mod leverage;
mod leverage_report;
mod naming;
mod shape_report;
mod signature;
mod syn_source;
mod tally;
mod trait_catalog;
mod trait_graph;
mod trait_key;
mod trait_shape;
mod type_key;

use std::fmt;

use concept_report::ConceptReport;
use fact_source::FactSource;
use formal_context::FormalContext;
use implementations::Implementations;
use leverage::Leverage;
use leverage_report::LeverageReport;
use shape_report::ShapeReport;
pub use syn_source::SynSource;
use tally::Tally;
use trait_catalog::TraitCatalog;
use trait_graph::TraitGraph;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Measures {
    types: Tally,
    traits: Tally,
    impls: Tally,
    concepts: ConceptReport,
    leverage: LeverageReport,
    graph: TraitGraph,
    shape: ShapeReport,
}

impl Measures {
    #[must_use]
    pub fn measured(source: &impl FactSource) -> Measures {
        let facts = source.facts();
        let catalog = TraitCatalog::cataloged(&facts);
        let implementations = Implementations::closed(&facts);
        let graph = TraitGraph::drawn(&facts);
        Measures {
            types: Tally::counting(facts.types()),
            traits: Tally::counting(facts.traits()),
            impls: Tally::counting(facts.impls()),
            concepts: ConceptReport::new(FormalContext::incidence(&facts).concepts(&catalog, &implementations)),
            leverage: LeverageReport::new(
                facts
                    .types()
                    .iter()
                    .map(|owner| Leverage::measured(owner, &facts, &catalog, &implementations))
                    .collect(),
            ),
            shape: ShapeReport::measured(&facts, &graph),
            graph,
        }
    }
}

impl fmt::Display for Measures {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "types {}, traits declared {}, impls {}", self.types, self.traits, self.impls)?;
        write!(formatter, "{}{}", self.concepts, self.leverage)?;
        writeln!(formatter, "trait-graph shape\n  {}", self.graph)?;
        write!(formatter, "{}", self.shape)
    }
}

#[cfg(test)]
mod tests {
    use super::{Measures, SynSource};
    use task::SourceFile;

    const CRATES: [[&str; 3]; 2] = [
        ["fixture", "fixture/src/lib.rs", include_str!("../fixture/shapes.rs.txt")],
        ["remote", "remote/src/lib.rs", include_str!("../fixture/remote.rs.txt")],
    ];
    const GOLDEN: &str = include_str!("../goldens/fixture.txt");

    #[test]
    fn the_fixture_measures_match_the_hand_worked_golden() {
        let files: Vec<SourceFile> = CRATES
            .iter()
            .map(|[owner, path, text]| {
                SourceFile::parse(*path, (*text).to_owned()).unwrap().in_crate(owner.parse().unwrap())
            })
            .collect();
        assert_eq!(Measures::measured(&SynSource::new(&files)).to_string(), GOLDEN);
    }
}
