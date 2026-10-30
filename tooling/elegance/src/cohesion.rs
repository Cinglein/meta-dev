use std::collections::BTreeSet;

use crate::components::Components;
use crate::facts::Facts;
use crate::field::Field;
use crate::field_name::FieldName;
use crate::finding::Finding;
use crate::findings::Findings;
use crate::layout::Layout;
use crate::method_use::MethodUse;
use crate::resolution::Resolution;
use crate::structure::Structure;
use crate::type_key::TypeKey;
use crate::usage_node::UsageNode;

#[derive(Debug)]
pub struct Cohesion;

impl Cohesion {
    pub const RULE: &'static str = "LCOM4 per type with fields, over its self methods keyed by trait and name; nodes are those methods and the fields; edges join a method to each field it reads or writes, to each self method it calls, and to the field of each accessor it calls; an accessor (body is self.f, &self.f or self.f = value) is not a node; a bare use of self joins every field; a method that touches nothing is left out; a self call that names no method or accessor of the type links nothing; on a type none of whose methods takes &mut self or writes a field and that has no Mutex, RwLock, RefCell, Cell, OnceCell or OnceLock field (a value or settings type), a method that reaches exactly one field is a projection and is left out";
    pub const LIMITS: &'static str = "syntactic: fields reached through helpers or locals and trait default bodies are not seen; a call is resolved by name to every method of the type with that name and, when several accessors share the name, to the field of each";

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
        let layout: Vec<&Field> = structure.layout(owner).into_iter().flat_map(Layout::fields).collect();
        let fields: BTreeSet<FieldName> = layout.iter().map(|field| field.name()).cloned().collect();
        let methods: Vec<&MethodUse> = structure.methods_of(owner).filter(|_| !fields.is_empty()).collect();
        let resolution = Resolution::new(fields, &methods);
        let value = Self::is_value(&layout, &methods);
        Components::joined(
            methods
                .iter()
                .filter(|method| method.accessed().is_none())
                .map(|method| (method, Self::reached(method, &resolution)))
                .filter(|(_, reached)| !(value && Self::is_projection(reached)))
                .filter_map(|(method, reached)| Self::neighbourhood(method, reached)),
        )
    }

    #[must_use]
    pub fn is_value(fields: &[&Field], methods: &[&MethodUse]) -> bool {
        fields.iter().all(|field| !field.is_shared()) && methods.iter().all(|method| !method.mutates())
    }

    #[must_use]
    pub fn is_projection(reached: &BTreeSet<UsageNode>) -> bool {
        matches!(reached.iter().collect::<Vec<_>>().as_slice(), [UsageNode::Field(_)])
    }

    #[must_use]
    pub fn reached(method: &MethodUse, resolution: &Resolution) -> BTreeSet<UsageNode> {
        let own = UsageNode::Own(method.key().clone());
        method
            .usage()
            .touched()
            .iter()
            .cloned()
            .flat_map(|node| node.resolved(resolution))
            .filter(|node| *node != own)
            .collect()
    }

    #[must_use]
    pub fn neighbourhood(method: &MethodUse, reached: BTreeSet<UsageNode>) -> Option<BTreeSet<UsageNode>> {
        Some(reached)
            .filter(|reached| !reached.is_empty())
            .map(|reached| reached.into_iter().chain([UsageNode::Own(method.key().clone())]).collect())
    }
}
