use std::collections::BTreeSet;

use crate::components::Components;
use crate::facts::Facts;
use crate::field::Field;
use crate::field_name::FieldName;
use crate::finding::Finding;
use crate::findings::Findings;
use crate::method_use::MethodUse;
use crate::resolution::Resolution;
use crate::structure::Structure;
use crate::type_key::TypeKey;
use crate::usage_node::UsageNode;

#[derive(Debug)]
pub struct Cohesion;

impl Cohesion {
    pub const RULE: &'static str = "LCOM4 per type with fields, over its self methods keyed by trait and name; nodes are those methods and the fields; edges join a method to each field it reads or writes, to each self method it calls, and to the field of each accessor it calls; an accessor (body is self.f, &self.f or self.f = value) is not a node; a bare use of self joins every field; a method that touches nothing is left out";
    pub const LIMITS: &'static str = "syntactic: fields reached through helpers or locals and trait default bodies are not seen; a call is resolved by name to every method of the type with that name";

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
        let fields: BTreeSet<FieldName> =
            structure.layout(owner).into_iter().flat_map(|layout| layout.fields().map(Field::name)).cloned().collect();
        let methods: Vec<&MethodUse> = structure.methods_of(owner).filter(|_| !fields.is_empty()).collect();
        let resolution = Resolution::new(fields, &methods);
        Components::joined(
            methods
                .iter()
                .filter(|method| method.accessed().is_none())
                .filter_map(|method| Self::neighbourhood(method, &resolution)),
        )
    }

    #[must_use]
    pub fn neighbourhood(method: &MethodUse, resolution: &Resolution) -> Option<BTreeSet<UsageNode>> {
        let own = UsageNode::Own(method.key().clone());
        let touched: BTreeSet<UsageNode> = method
            .usage()
            .touched()
            .iter()
            .cloned()
            .flat_map(|node| node.resolved(resolution))
            .filter(|node| *node != own)
            .collect();
        Some(touched).filter(|touched| !touched.is_empty()).map(|touched| touched.into_iter().chain([own]).collect())
    }
}
