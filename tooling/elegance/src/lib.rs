mod concept;
mod concept_report;
mod distribution;
mod elegance_settings;
mod fact_source;
mod facts;
mod formal_context;
mod hundredths;
mod impl_fact;
mod impl_kind;
mod implementations;
mod intent_cap;
mod leverage;
mod leverage_report;
mod method_name;
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
pub use elegance_settings::EleganceSettings;
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
    pub fn measured(source: &impl FactSource, settings: &EleganceSettings) -> Measures {
        let facts = source.facts();
        let catalog = TraitCatalog::cataloged(&facts);
        let implementations = Implementations::closed(&facts);
        let graph = TraitGraph::drawn(&facts);
        Measures {
            types: Tally::counting(facts.types()),
            traits: Tally::counting(facts.traits()),
            impls: Tally::counting(facts.impls()),
            concepts: FormalContext::incidence(&facts).concepts(&catalog, &implementations, settings.max_intents()),
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
    use super::{EleganceSettings, Measures, SynSource};
    use meta_golden::Golden;
    use settings::EmbeddedDefaults;
    use task::SourceFile;

    const CRATES: [[&str; 3]; 2] = [
        ["fixture", "fixture/src/lib.rs", include_str!("../fixture/shapes.rs.txt")],
        ["remote", "remote/src/lib.rs", include_str!("../fixture/remote.rs.txt")],
    ];
    const SMALL_CAP: &str = "max_intents = 4";
    const INCOMPLETE: &str = "INCOMPLETE: more than 4 distinct intents";
    const SNAPSHOT: &str = "fixture";

    struct Fixture;

    impl Fixture {
        fn measured(settings: EleganceSettings) -> String {
            let files: Vec<SourceFile> = CRATES
                .iter()
                .map(|[owner, path, text]| {
                    SourceFile::parse(*path, (*text).to_owned()).unwrap().in_crate(owner.parse().unwrap())
                })
                .collect();
            Measures::measured(&SynSource::new(&files), &settings).to_string()
        }
    }

    #[test]
    fn the_fixture_measures_match_the_golden() {
        let measured = Fixture::measured(EleganceSettings::defaults());
        Golden::for_test(module_path!(), SNAPSHOT).checked(&measured).unwrap();
    }

    #[test]
    fn a_hit_intent_cap_is_reported_loudly_and_the_default_cap_is_not_hit() {
        let capped: EleganceSettings = toml::from_str(SMALL_CAP).unwrap();
        let (hit, spared) = (Fixture::measured(capped), Fixture::measured(EleganceSettings::defaults()));
        assert!(hit.contains(INCOMPLETE) && !spared.contains(INCOMPLETE), "{hit}");
    }
}
