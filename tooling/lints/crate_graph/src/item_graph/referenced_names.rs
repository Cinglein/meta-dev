use std::collections::BTreeMap;

use quote::ToTokens;
use syn::{Item, Type};

use task::Name;
use task::Walk;

use crate::item_graph::scope::Scope;

#[derive(Debug, Default)]
pub struct ReferencedNames(BTreeMap<Name, Vec<Name>>);

impl ReferencedNames {
    pub fn of_items(items: &[Item], scope: &Scope) -> ReferencedNames {
        items
            .iter()
            .fold(ReferencedNames::default(), |names, item| {
                names.with_item(item, scope)
            })
    }

    pub fn into_map(self) -> BTreeMap<Name, Vec<Name>> {
        self.0
    }

    fn with_item(mut self, item: &Item, scope: &Scope) -> ReferencedNames {
        let owner = match item {
            Item::Struct(item) if Walk::outside_tests(&item.attrs) == Walk::Entered => {
                Some(Name::from(&item.ident))
            }
            Item::Enum(item) if Walk::outside_tests(&item.attrs) == Walk::Entered => {
                Some(Name::from(&item.ident))
            }
            Item::Trait(item) if Walk::outside_tests(&item.attrs) == Walk::Entered => {
                Some(Name::from(&item.ident))
            }
            Item::Fn(item) if Walk::outside_tests(&item.attrs) == Walk::Entered => {
                Some(Name::from(&item.sig.ident))
            }
            Item::Type(item) if Walk::outside_tests(&item.attrs) == Walk::Entered => {
                Some(Name::from(&item.ident))
            }
            Item::Const(item) if Walk::outside_tests(&item.attrs) == Walk::Entered => {
                Some(Name::from(&item.ident))
            }
            Item::Static(item) if Walk::outside_tests(&item.attrs) == Walk::Entered => {
                Some(Name::from(&item.ident))
            }
            Item::Impl(item) if Walk::outside_tests(&item.attrs) == Walk::Entered => {
                Self::self_type(&item.self_ty)
            }
            _ => None,
        };
        let nested = match item {
            Item::Mod(module) if Walk::outside_tests(&module.attrs) == Walk::Entered => module
                .content
                .as_ref()
                .map(|(_, items)| Self::of_items(items, scope).0)
                .unwrap_or_default(),
            _ => BTreeMap::new(),
        };
        owner
            .into_iter()
            .map(|owner| (owner, scope.referenced(item.to_token_stream())))
            .chain(nested)
            .fold(&mut self.0, |names, (owner, found)| {
                names.entry(owner).or_default().extend(found);
                names
            });
        self
    }

    fn self_type(self_type: &Type) -> Option<Name> {
        match self_type {
            Type::Path(path) => path
                .path
                .segments
                .last()
                .map(|segment| Name::from(&segment.ident)),
            _ => None,
        }
    }
}
