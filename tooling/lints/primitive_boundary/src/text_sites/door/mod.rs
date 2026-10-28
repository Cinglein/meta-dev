mod door_kind;

use std::path::Path;

use door_kind::DoorKind;
use proc_macro2::TokenStream;
use syn::{AngleBracketedGenericArguments, GenericArgument, Ident, ItemImpl, Type};

use task::Name;
use task::Site;

pub struct Door {
    kind: DoorKind,
    target: Option<Name>,
    site: Option<Site>,
}

impl Door {
    pub fn of_impl(item: &ItemImpl, site: Site) -> Option<Door> {
        DoorKind::of_impl(item).map(|kind| Door {
            kind,
            target: Self::target_of(&item.self_ty),
            site: Some(site),
        })
    }

    pub fn of_callee(callee: &Ident, target: Option<Name>, within: Option<Site>) -> Option<Door> {
        DoorKind::of_callee(callee).map(|kind| Door {
            kind,
            target,
            site: within,
        })
    }

    pub fn in_tokens(tokens: TokenStream, within: Option<&Site>) -> Vec<Door> {
        DoorKind::in_tokens(tokens)
            .into_iter()
            .map(|kind| Door {
                kind,
                target: None,
                site: within.cloned(),
            })
            .collect()
    }

    pub fn target_of(ty: &Type) -> Option<Name> {
        match ty {
            Type::Path(path) => path
                .path
                .segments
                .last()
                .map(|segment| Name::from(&segment.ident)),
            _ => None,
        }
    }

    pub fn first_type(arguments: Option<&AngleBracketedGenericArguments>) -> Option<Name> {
        arguments
            .and_then(|arguments| arguments.args.first())
            .and_then(|argument| match argument {
                GenericArgument::Type(ty) => Self::target_of(ty),
                _ => None,
            })
    }

    pub fn is_in(&self, path: &Path) -> bool {
        matches!(&self.site, Some(Site::Line(file, _)) if file == path)
    }

    pub fn is_reached_by(&self, seen: &Door) -> bool {
        self.kind == seen.kind
            && seen.site != self.site
            && seen
                .target
                .as_ref()
                .is_none_or(|target| self.target.as_ref().is_none_or(|own| own == target))
    }
}
