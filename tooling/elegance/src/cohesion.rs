use std::collections::BTreeSet;

use crate::components::Components;
use crate::facts::Facts;
use crate::field::Field;
use crate::finding::Finding;
use crate::findings::Findings;
use crate::method_use::MethodUse;
use crate::structure::Structure;
use crate::tally::Tally;
use crate::type_key::TypeKey;
use crate::usage_node::UsageNode;

#[derive(Debug)]
pub struct Cohesion;

impl Cohesion {
    pub const RULE: &'static str = "LCOM4 per type, over its self methods that are not accessors (an accessor touches exactly one field and nothing else): nodes are those methods (all impl blocks) and the fields; edges join a method to each field it reads or writes and to each self method it calls; a bare use of self joins every field";
    pub const LIMITS: &'static str = "a method that touches no field and calls no self method is its own component; trait default bodies and fields reached through helpers are not seen";

    #[must_use]
    pub fn measured(facts: &Facts) -> Findings<TypeKey, Components> {
        let structure = facts.structure();
        Findings::new(facts.types().iter().filter_map(|owner| {
            Some(Self::components(structure, owner))
                .filter(Components::is_split)
                .map(|components| Finding::new(owner.clone(), components))
        }))
        .sorted_by(|first, second| first.detail().most_first(second.detail()))
    }

    #[must_use]
    pub fn components(structure: &Structure, owner: &TypeKey) -> Components {
        let fields: BTreeSet<UsageNode> = structure
            .layout(owner)
            .into_iter()
            .flat_map(|layout| layout.fields().map(Field::name))
            .map(|name| UsageNode::Field(name.clone()))
            .collect();
        let methods = structure.methods_of(owner).filter(|method| !Self::is_accessor(method));
        Components::joined(methods.map(|method| Self::neighbourhood(method, &fields)))
    }

    #[must_use]
    pub fn is_accessor(method: &MethodUse) -> bool {
        let touched = method.usage().touched();
        Tally::counting(touched) == Tally::ONE && touched.iter().all(|node| node.field().is_some())
    }

    #[must_use]
    pub fn neighbourhood(method: &MethodUse, fields: &BTreeSet<UsageNode>) -> BTreeSet<UsageNode> {
        let touched = method.usage().touched();
        let whole = touched.contains(&UsageNode::Whole).then_some(fields).into_iter().flatten();
        std::iter::once(UsageNode::Method(method.name().clone()))
            .chain(touched.iter().cloned())
            .chain(whole.cloned())
            .collect()
    }
}
