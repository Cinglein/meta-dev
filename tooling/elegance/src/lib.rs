mod catalog;
mod cohesion;
mod components;
mod concept;
mod concept_report;
mod distribution;
mod elegance_settings;
mod fact_source;
mod facts;
mod field;
mod field_kind;
mod field_name;
mod field_write;
mod finding;
mod findings;
mod formal_context;
mod hidden_field;
mod hidden_state;
mod hundredths;
mod impl_fact;
mod impl_kind;
mod implementations;
mod intent_cap;
mod interior;
mod layout;
mod leverage;
mod leverage_report;
mod listed;
mod located;
mod method_key;
mod method_name;
mod method_use;
mod naming;
mod places;
mod receiver_kind;
mod reducible_traits;
mod reduction;
mod resolution;
mod scoring;
mod shape_report;
mod shared_signatures;
mod signature;
mod smell_inventory;
mod source_path;
mod structure;
mod syn_source;
mod tally;
mod trait_catalog;
mod trait_graph;
mod trait_key;
mod trait_shape;
mod type_key;
mod usage;
mod usage_node;
mod write_kind;

use std::fmt;

use catalog::Catalog;
use cohesion::Cohesion;
use components::Components;
use concept_report::ConceptReport;
pub use elegance_settings::EleganceSettings;
use fact_source::FactSource;
use field::Field;
use findings::Findings;
use formal_context::FormalContext;
use hidden_field::HiddenField;
use hidden_state::HiddenState;
use implementations::Implementations;
use leverage::Leverage;
use leverage_report::LeverageReport;
use listed::Listed;
use method_key::MethodKey;
use places::Places;
use reducible_traits::ReducibleTraits;
use reduction::Reduction;
pub use scoring::Scoring;
use shape_report::ShapeReport;
use shared_signatures::SharedSignatures;
use signature::Signature;
pub use smell_inventory::SmellInventory;
pub use syn_source::SynSource;
use tally::Tally;
use trait_catalog::TraitCatalog;
use trait_graph::TraitGraph;
use trait_key::TraitKey;
use type_key::TypeKey;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Measures {
    types: Tally,
    traits: Tally,
    impls: Tally,
    concepts: ConceptReport<TypeKey, Signature>,
    reducible: Findings<TraitKey, Reduction<TraitKey>>,
    shared: Findings<Signature, Listed<TraitKey>>,
    leverage: LeverageReport,
    hidden: Findings<HiddenField, Listed<MethodKey>>,
    fields: ConceptReport<TypeKey, Field>,
    split: Findings<TypeKey, Components>,
    graph: TraitGraph,
    shape: ShapeReport,
    places: Places,
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
            concepts: FormalContext::incidence(&facts)
                .concepts(settings.max_intents())
                .named(&catalog, &implementations),
            reducible: ReducibleTraits::measured(&facts, &implementations),
            shared: SharedSignatures::measured(&facts),
            leverage: LeverageReport::new(
                facts
                    .types()
                    .iter()
                    .map(|owner| Leverage::measured(owner, &facts, &catalog, &implementations))
                    .collect(),
            ),
            hidden: HiddenState::measured(&facts),
            fields: FormalContext::fields(&facts).concepts(settings.max_intents()),
            split: Cohesion::measured(&facts),
            shape: ShapeReport::measured(&facts, &graph),
            places: facts.structure().places().clone(),
            graph,
        }
    }
}

impl fmt::Display for Measures {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "types {}, traits declared {}, impls {}", self.types, self.traits, self.impls)?;
        Catalog::listed(self, formatter)?;
        write!(formatter, "{}", self.concepts)?;
        write!(
            formatter,
            "B2 reducible traits: same implementors as another trait, or exactly the meet of two or more\n  local traits only; a trait with a blanket impl is derived, not reducible\n{}",
            self.reducible
        )?;
        write!(formatter, "B3 signatures declared in more than one local trait\n{}", self.shared)?;
        write!(formatter, "B4 {}", self.leverage)?;
        write!(
            formatter,
            "B9 hidden state\n  rule: {}\n  limits: {}\n{}",
            HiddenState::RULE,
            HiddenState::LIMITS,
            self.hidden
        )?;
        write!(formatter, "{}", self.fields)?;
        write!(
            formatter,
            "B13 types whose method-field graph splits\n  rule: {}\n  limits: {}\n{}",
            Cohesion::RULE,
            Cohesion::LIMITS,
            self.split
        )?;
        writeln!(formatter, "trait-graph shape\n  {}", self.graph)?;
        write!(formatter, "{}", self.shape)
    }
}

#[cfg(test)]
mod tests {
    use super::{Catalog, EleganceSettings, Measures, Scoring, SmellInventory, SynSource};
    use std::sync::OnceLock;

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
    const SMELLS: &str = include_str!("../fixture/smells.toml");

    struct Fixture;

    impl Fixture {
        fn measured(settings: EleganceSettings) -> String {
            let files: Vec<SourceFile> = CRATES
                .iter()
                .map(|[owner, path, text]| {
                    SourceFile::parse(*path, (*text).to_owned()).unwrap().in_crate(owner.parse().unwrap())
                })
                .collect();
            let measures = Measures::measured(&SynSource::new(&files), &settings);
            let smells: SmellInventory = toml::from_str(SMELLS).unwrap();
            format!("{measures}{}", Scoring::new(&measures, &smells))
        }

        fn by_default() -> &'static str {
            static MEASURED: OnceLock<String> = OnceLock::new();
            MEASURED.get_or_init(|| Self::measured(EleganceSettings::defaults()))
        }
    }

    #[test]
    fn the_fixture_measures_match_the_golden() {
        Golden::for_test(module_path!(), SNAPSHOT).checked(Fixture::by_default()).unwrap();
    }

    #[test]
    fn a_hit_intent_cap_is_reported_loudly_and_the_default_cap_is_not_hit() {
        let capped: EleganceSettings = toml::from_str(SMALL_CAP).unwrap();
        let (hit, spared) = (Fixture::measured(capped), Fixture::by_default());
        assert!(hit.contains(INCOMPLETE) && !spared.contains(INCOMPLETE), "{hit}");
    }

    #[test]
    fn the_catalog_lists_each_measure_once_in_order() {
        let lines: Vec<String> = Catalog::MEASURES.iter().map(ToString::to_string).collect();
        let misplaced: Vec<&String> = (1..=lines.len())
            .zip(&lines)
            .filter(|(number, line)| !line.starts_with(&format!("B{number} ")))
            .map(|(_, line)| line)
            .collect();
        assert!(misplaced.is_empty(), "{misplaced:?}");
    }
}
