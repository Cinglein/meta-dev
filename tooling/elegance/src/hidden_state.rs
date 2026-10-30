use crate::facts::Facts;
use crate::field::Field;
use crate::finding::Finding;
use crate::findings::Findings;
use crate::hidden_field::HiddenField;
use crate::layout::Layout;
use crate::listed::Listed;
use crate::method_key::MethodKey;
use crate::method_use::MethodUse;
use crate::receiver_kind::ReceiverKind;
use crate::structure::Structure;
use crate::type_key::TypeKey;

#[derive(Debug)]
pub struct HiddenState;

impl HiddenState {
    pub const RULE: &'static str = "a bool or local-enum field assigned (= or op=), or an Option field assigned or filled (insert, replace, take, get_or_insert, get_or_insert_with), inside a &mut self method of the same type";
    pub const LIMITS: &'static str = "syntactic: misses writes through aliases (let f = &mut self.f), mem::replace or swap, helper fns, other types' methods, type aliases of bool or Option, and enums from other crates";

    #[must_use]
    pub fn measured(facts: &Facts) -> Findings<HiddenField, Listed<MethodKey>> {
        let structure = facts.structure();
        Findings::new(facts.types().iter().flat_map(|owner| {
            let fields = structure.layout(owner).into_iter().flat_map(Layout::fields);
            fields.filter_map(|field| Self::written(structure, owner, field))
        }))
    }

    #[must_use]
    pub fn written(
        structure: &Structure,
        owner: &TypeKey,
        field: &Field,
    ) -> Option<Finding<HiddenField, Listed<MethodKey>>> {
        let writers = Listed::new(
            structure.methods_of(owner).filter(|method| Self::hides(method, field)).map(MethodUse::key).cloned(),
        );
        Some(writers)
            .filter(|writers| !writers.is_empty())
            .map(|writers| Finding::new(HiddenField::new(owner.clone(), field.clone()), writers))
    }

    #[must_use]
    pub fn hides(method: &MethodUse, field: &Field) -> bool {
        let hiding = field.kind().hiding_writes();
        method.receiver() == ReceiverKind::Exclusive
            && method
                .usage()
                .writes()
                .iter()
                .any(|write| write.field() == field.name() && hiding.contains(&write.how()))
    }
}
