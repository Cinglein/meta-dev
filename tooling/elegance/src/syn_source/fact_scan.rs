use std::collections::{BTreeMap, BTreeSet};

use syn::visit::Visit;
use syn::{
    Ident, ImplItem, ItemFn, ItemImpl, ItemMod, ItemTrait, TraitBoundModifier, TraitItem, Type, TypeParamBound,
    WherePredicate,
};
use task::{CrateName, SourceFile, Walk};

use super::declarations::Declarations;
use crate::facts::Facts;
use crate::impl_fact::ImplFact;
use crate::impl_kind::ImplKind;
use crate::signature::Signature;
use crate::trait_key::TraitKey;
use crate::trait_shape::TraitShape;
use crate::type_key::TypeKey;

#[derive(Debug)]
pub struct FactScan<'declared> {
    declarations: &'declared Declarations,
    owner: Option<CrateName>,
    traits: BTreeMap<TraitKey, TraitShape>,
    impls: Vec<ImplFact>,
}

impl<'declared> FactScan<'declared> {
    #[must_use]
    pub fn scanned(declarations: &'declared Declarations, files: &[&SourceFile]) -> Facts {
        let empty = FactScan { declarations, owner: None, traits: BTreeMap::new(), impls: Vec::new() };
        let scan = files.iter().fold(empty, |scan, file| scan.within(file));
        Facts::new(declarations.local_types(), scan.traits, scan.impls)
    }

    fn within(mut self, file: &SourceFile) -> FactScan<'declared> {
        self.owner = file.crate_name().cloned();
        self.visit_file(file.syntax());
        self
    }

    #[must_use]
    pub fn named(ty: &Type) -> Option<&Ident> {
        match ty {
            Type::Path(path) => path.path.segments.last().map(|segment| &segment.ident),
            Type::Reference(reference) => Self::named(&reference.elem),
            Type::Paren(paren) => Self::named(&paren.elem),
            Type::Group(group) => Self::named(&group.elem),
            _ => None,
        }
    }

    #[must_use]
    pub fn blanket_parameter(item: &ItemImpl) -> Option<&Ident> {
        match &*item.self_ty {
            Type::Path(path) if path.qself.is_none() => {
                path.path.get_ident().filter(|ident| Self::generic(item, ident))
            }
            _ => None,
        }
    }

    #[must_use]
    pub fn generic(item: &ItemImpl, ident: &Ident) -> bool {
        item.generics.type_params().any(|param| param.ident == *ident)
    }

    #[must_use]
    pub fn trait_keys<'bound>(
        &self,
        owner: &CrateName,
        bounds: impl Iterator<Item = &'bound TypeParamBound>,
    ) -> BTreeSet<TraitKey> {
        bounds
            .filter_map(|bound| match bound {
                TypeParamBound::Trait(traited) if matches!(traited.modifier, TraitBoundModifier::None) => {
                    traited.path.segments.last()
                }
                _ => None,
            })
            .map(|segment| self.declarations.trait_key(owner, &segment.ident))
            .collect()
    }

    #[must_use]
    pub fn bounds(&self, owner: &CrateName, item: &ItemImpl, parameter: &Ident) -> BTreeSet<TraitKey> {
        let declared =
            item.generics.type_params().filter(|param| param.ident == *parameter).flat_map(|param| param.bounds.iter());
        let clauses = item
            .generics
            .where_clause
            .iter()
            .flat_map(|clause| clause.predicates.iter())
            .filter_map(|predicate| match predicate {
                WherePredicate::Type(typed) if Self::named(&typed.bounded_ty) == Some(parameter) => {
                    Some(typed.bounds.iter())
                }
                _ => None,
            })
            .flatten();
        self.trait_keys(owner, declared.chain(clauses))
    }

    #[must_use]
    pub fn kind(&self, owner: &CrateName, item: &ItemImpl) -> Option<ImplKind> {
        let implemented = item
            .trait_
            .as_ref()
            .and_then(|(_, path, _)| path.segments.last())
            .map(|segment| self.declarations.trait_key(owner, &segment.ident));
        let target = Self::named(&item.self_ty)
            .map_or_else(|| TypeKey::shaped(&item.self_ty), |ident| self.declarations.type_key(owner, ident));
        match (implemented, Self::blanket_parameter(item)) {
            (None, _) => Some(ImplKind::Inherent(target)),
            (Some(implemented), Some(parameter)) => {
                Some(ImplKind::Blanket { bounds: self.bounds(owner, item, parameter), implemented })
            }
            (Some(implemented), None) => Some(ImplKind::Concrete { implemented, target }),
        }
        .filter(|_| item.trait_.as_ref().is_none_or(|(negated, _, _)| negated.is_none()))
    }

    #[must_use]
    pub fn fact(&self, owner: &CrateName, item: &ItemImpl) -> Option<ImplFact> {
        let own = Self::blanket_parameter(item).or_else(|| Self::named(&item.self_ty));
        self.kind(owner, item).map(|kind| ImplFact::new(kind, Self::bodies(item, own)))
    }

    #[must_use]
    pub fn bodies(item: &ItemImpl, own: Option<&Ident>) -> Vec<Signature> {
        item.items
            .iter()
            .filter_map(|member| match member {
                ImplItem::Fn(function) => Some(Signature::abstracted(&function.sig, own)),
                _ => None,
            })
            .collect()
    }

    #[must_use]
    pub fn shape(&self, owner: &CrateName, item: &ItemTrait) -> TraitShape {
        TraitShape::new(
            item.items
                .iter()
                .filter_map(|member| match member {
                    TraitItem::Fn(function) => Some(Signature::abstracted(&function.sig, None)),
                    _ => None,
                })
                .collect(),
            self.trait_keys(owner, item.supertraits.iter()),
        )
    }
}

impl<'ast> Visit<'ast> for FactScan<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        match SourceFile::walk(module) {
            Walk::Entered => syn::visit::visit_item_mod(self, module),
            Walk::Skipped => {}
        }
    }

    fn visit_item_fn(&mut self, _: &'ast ItemFn) {}

    fn visit_item_trait(&mut self, item: &'ast ItemTrait) {
        let shaped =
            self.owner.as_ref().map(|owner| (self.declarations.trait_key(owner, &item.ident), self.shape(owner, item)));
        self.traits.extend(shaped);
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        let fact = self.owner.as_ref().and_then(|owner| self.fact(owner, item));
        self.impls.extend(fact);
    }
}
