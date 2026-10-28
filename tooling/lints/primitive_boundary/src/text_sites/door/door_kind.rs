use proc_macro2::{TokenStream, TokenTree};
use syn::{Ident, ItemImpl, Lit};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DoorKind {
    FromStr,
    TryFrom,
    Display,
}

impl DoorKind {
    const PLACEHOLDER: char = '{';

    pub fn of_impl(item: &ItemImpl) -> Option<DoorKind> {
        item.trait_
            .as_ref()
            .and_then(|(_, path, _)| path.segments.last())
            .and_then(|segment| match &segment.ident {
                ident if ident == "FromStr" => Some(DoorKind::FromStr),
                ident if ident == "TryFrom" => Some(DoorKind::TryFrom),
                ident if ident == "Display" => Some(DoorKind::Display),
                _ => None,
            })
    }

    pub fn of_callee(ident: &Ident) -> Option<DoorKind> {
        match ident {
            ident if ident == "from_str" || ident == "parse" => Some(DoorKind::FromStr),
            ident if ident == "try_from" || ident == "try_into" => Some(DoorKind::TryFrom),
            ident if ident == "to_string" => Some(DoorKind::Display),
            _ => None,
        }
    }

    pub fn in_tokens(tokens: TokenStream) -> Vec<DoorKind> {
        tokens
            .into_iter()
            .flat_map(|tree| match tree {
                TokenTree::Literal(literal) => matches!(Lit::new(literal), Lit::Str(text) if text.value().contains(Self::PLACEHOLDER))
                    .then_some(DoorKind::Display)
                    .into_iter()
                    .collect(),
                TokenTree::Ident(ident) => Self::of_callee(&ident).into_iter().collect(),
                TokenTree::Group(group) => Self::in_tokens(group.stream()),
                TokenTree::Punct(_) => Vec::new(),
            })
            .collect()
    }
}
