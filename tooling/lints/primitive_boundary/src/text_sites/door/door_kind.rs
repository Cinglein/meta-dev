use std::str::FromStr;

use proc_macro2::{TokenStream, TokenTree};
use syn::{Ident, ItemImpl, Lit};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DoorKind {
    FromStr,
    TryFrom,
    Display,
}

impl DoorKind {
    pub const EVERY: [DoorKind; 3] = [DoorKind::FromStr, DoorKind::TryFrom, DoorKind::Display];
    const OTHER_TRAITS: &str = "?xXobeEp";
    const ESCAPED: &str = "{{";
    const OPEN: char = '{';
    const CLOSE: char = '}';
    const SPEC: char = ':';
    const SIGIL: char = '%';

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

    pub fn in_known(tokens: TokenStream) -> Vec<DoorKind> {
        tokens.into_iter().flat_map(Self::in_known_token).collect()
    }

    pub fn in_unknown(tokens: &TokenStream) -> Vec<DoorKind> {
        tokens
            .clone()
            .into_iter()
            .flat_map(|tree| Self::in_unknown_token(&tree))
            .collect()
    }

    fn in_known_token(tree: TokenTree) -> Vec<DoorKind> {
        match tree {
            TokenTree::Literal(literal) => Self::of_literal(Lit::new(literal)),
            TokenTree::Ident(ident) => Self::of_callee(&ident).into_iter().collect(),
            TokenTree::Group(group) => Self::in_known(group.stream()),
            TokenTree::Punct(_) => Vec::new(),
        }
    }

    fn in_unknown_token(tree: &TokenTree) -> Vec<DoorKind> {
        match tree {
            TokenTree::Punct(punct) if punct.as_char() == Self::SIGIL => vec![DoorKind::Display],
            TokenTree::Group(group) => Self::in_unknown(&group.stream()),
            other => Self::widened(&Self::in_known_token(other.clone())),
        }
    }

    fn of_literal(literal: Lit) -> Vec<DoorKind> {
        match literal {
            Lit::Str(text) => text.value().parse::<DoorKind>().into_iter().collect(),
            _ => Vec::new(),
        }
    }

    fn widened(kinds: &[DoorKind]) -> Vec<DoorKind> {
        match kinds {
            [] => Vec::new(),
            _ => Self::EVERY.to_vec(),
        }
    }
}

impl FromStr for DoorKind {
    type Err = ();

    fn from_str(text: &str) -> Result<DoorKind, ()> {
        text.replace(Self::ESCAPED, "")
            .split(Self::OPEN)
            .skip(1)
            .filter_map(|piece| piece.split_once(Self::CLOSE))
            .any(|(inside, _)| {
                inside
                    .split_once(Self::SPEC)
                    .map_or("", |(_, spec)| spec)
                    .chars()
                    .last()
                    .is_none_or(|last| !Self::OTHER_TRAITS.contains(last))
            })
            .then_some(DoorKind::Display)
            .ok_or(())
    }
}
