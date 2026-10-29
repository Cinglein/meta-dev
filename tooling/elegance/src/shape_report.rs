use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::distribution::Distribution;
use crate::facts::Facts;
use crate::implementations::Implementations;
use crate::tally::Tally;
use crate::trait_graph::TraitGraph;
use crate::trait_key::TraitKey;
use crate::type_key::TypeKey;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShapeReport {
    trait_impls: Tally,
    blanket_impls: Tally,
    single: BTreeMap<TraitKey, TypeKey>,
    implemented: Distribution<Tally>,
    floor: Tally,
    unlinked: BTreeMap<TypeKey, BTreeSet<TraitKey>>,
}

impl ShapeReport {
    #[must_use]
    pub fn measured(facts: &Facts, graph: &TraitGraph) -> ShapeReport {
        let direct: BTreeMap<TypeKey, BTreeSet<TraitKey>> = facts
            .types()
            .iter()
            .map(|owner| (owner.clone(), Implementations::direct(facts, owner)))
            .filter(|(_, traits)| !traits.is_empty())
            .collect();
        let implemented = Distribution::sorted(direct.values().map(Tally::counting));
        let floor = implemented.quarter(Tally::UPPER_QUARTILE).copied().unwrap_or_default().max(Tally::PAIR);
        ShapeReport {
            trait_impls: Tally::counting(facts.impls().iter().filter_map(|fact| fact.kind().implemented())),
            blanket_impls: Tally::counting(facts.impls().iter().filter_map(|fact| fact.kind().blanket_bounds())),
            single: facts
                .traits()
                .keys()
                .filter_map(|key| Self::sole_implementor(facts, key).map(|target| (key.clone(), target)))
                .collect(),
            unlinked: direct
                .into_iter()
                .filter(|(_, traits)| Tally::counting(traits).reaches(floor) && !graph.linked(traits))
                .collect(),
            implemented,
            floor,
        }
    }

    #[must_use]
    pub fn sole_implementor(facts: &Facts, key: &TraitKey) -> Option<TypeKey> {
        let blanketed = facts.impls_of(key).filter_map(|fact| fact.kind().blanket_bounds());
        Some(facts.concrete_targets(key))
            .filter(|targets| Tally::counting(targets) == Tally::ONE)
            .filter(|_| Tally::counting(blanketed) == Tally::default())
            .and_then(|targets| targets.first().copied().cloned())
    }
}

impl fmt::Display for ShapeReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            formatter,
            "  blanket share: {} of {} trait impls = {}",
            self.blanket_impls,
            self.trait_impls,
            self.blanket_impls.per(self.trait_impls)
        )?;
        writeln!(formatter, "  single-implementor traits: {}", self.single.len())?;
        self.single.iter().try_for_each(|(key, target)| writeln!(formatter, "    {key} -> {target}"))?;
        writeln!(formatter, "  traits implemented directly, per implementing type: {}", self.implemented)?;
        writeln!(
            formatter,
            "  types with at least N = {} traits (q3, at least 2) and no edge among them: {}",
            self.floor,
            self.unlinked.len()
        )?;
        self.unlinked.iter().take(Tally::SHOWN).try_for_each(|(owner, traits)| {
            let names: Vec<String> = traits.iter().map(ToString::to_string).collect();
            writeln!(formatter, "    {owner}: {}", names.join(", "))
        })
    }
}
