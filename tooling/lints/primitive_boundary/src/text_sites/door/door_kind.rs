use proc_macro2::{TokenStream, TokenTree};
use syn::punctuated::Punctuated;
use syn::{Attribute, Ident, ItemImpl, Lit, Macro, Meta, Path, Token};

use task::{Name, Whole};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DoorKind {
    FromStr,
    TryFrom,
    Display,
}

impl DoorKind {
    const PLACEHOLDER: char = '{';
    const EVERY: [DoorKind; 3] = [DoorKind::FromStr, DoorKind::TryFrom, DoorKind::Display];
    const STD_DERIVES: [&str; 9] = [
        "Debug", "Clone", "Copy", "PartialEq", "Eq", "PartialOrd", "Ord", "Hash", "Default",
    ];
    const KNOWN_ATTRIBUTES: [&str; 22] = [
        "cfg", "cfg_attr", "test", "allow", "warn", "deny", "expect", "forbid", "must_use", "doc",
        "inline", "path", "repr", "non_exhaustive", "error", "rustfmt", "clippy", "diagnostic",
        "track_caller", "cold", "ignore", "should_panic",
    ];
    const KNOWN_MACROS: [&str; 33] = [
        "format", "write", "writeln", "print", "println", "eprint", "eprintln", "panic", "assert",
        "assert_eq", "assert_ne", "debug_assert", "debug_assert_eq", "debug_assert_ne",
        "format_args", "unreachable", "todo", "unimplemented", "vec", "matches", "concat",
        "stringify", "include_str", "include_bytes", "env", "option_env", "line", "column", "file",
        "module_path", "cfg", "compile_error", "dbg",
    ];
    const DERIVE: &str = "derive";

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

    pub fn of_macro(invocation: &Macro) -> Vec<DoorKind> {
        Self::unless_known(
            &invocation.path,
            &Self::KNOWN_MACROS,
            Self::in_tokens(invocation.tokens.clone()),
        )
    }

    pub fn of_attribute(attribute: &Attribute) -> Vec<DoorKind> {
        let name = attribute
            .path()
            .segments
            .first()
            .map(|segment| Name::from(&segment.ident))
            .unwrap_or_default();
        let tokens = match &attribute.meta {
            Meta::List(list) => list.tokens.clone(),
            Meta::Path(_) | Meta::NameValue(_) => TokenStream::new(),
        };
        match name {
            name if name == Self::DERIVE => Self::of_derive(
                attribute.parse_args_with(Punctuated::<Path, Token![,]>::parse_terminated),
            ),
            name if name.matches::<Whole>(&Self::KNOWN_ATTRIBUTES) => Self::in_tokens(tokens),
            _ => Self::EVERY.to_vec(),
        }
    }

    fn of_derive(derived: syn::Result<Punctuated<Path, Token![,]>>) -> Vec<DoorKind> {
        derived.map_or_else(
            |_| Self::EVERY.to_vec(),
            |derived| {
                derived
                    .iter()
                    .flat_map(|path| Self::unless_known(path, &Self::STD_DERIVES, Vec::new()))
                    .collect()
            },
        )
    }

    fn unless_known(path: &Path, names: &[&str], known: Vec<DoorKind>) -> Vec<DoorKind> {
        match path.segments.last() {
            Some(segment) if Name::from(&segment.ident).matches::<Whole>(names) => known,
            _ => Self::EVERY.to_vec(),
        }
    }

    fn in_tokens(tokens: TokenStream) -> Vec<DoorKind> {
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
