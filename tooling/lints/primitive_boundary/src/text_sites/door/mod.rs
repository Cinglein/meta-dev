mod door_kind;

use std::path::Path;

use door_kind::DoorKind;
use proc_macro2::{TokenStream, TokenTree};
use quote::ToTokens;
use syn::punctuated::Punctuated;
use syn::{
    AngleBracketedGenericArguments, Attribute, GenericArgument, Ident, ItemImpl, Macro, Meta,
    Token, Type,
};

use task::Site;
use task::{Name, Whole};

#[derive(Clone)]
pub struct Door {
    kind: DoorKind,
    target: Option<Name>,
    site: Option<Site>,
}

impl Door {
    const STD_TYPES: [&str; 41] = [
        "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize",
        "f32", "f64", "bool", "char", "String", "str", "Path", "PathBuf", "OsStr", "OsString",
        "IpAddr", "Ipv4Addr", "Ipv6Addr", "SocketAddr", "Duration", "Option", "Vec", "Box",
        "Result", "Rc", "Arc", "Cow", "BTreeMap", "BTreeSet", "HashMap", "HashSet", "VecDeque",
        "NonZeroU32", "NonZeroU64",
    ];
    const STD_DERIVES: [&str; 9] = [
        "Debug", "Clone", "Copy", "PartialEq", "Eq", "PartialOrd", "Ord", "Hash", "Default",
    ];
    const SILENT_ATTRIBUTES: [&str; 20] = [
        "cfg", "cfg_attr", "test", "allow", "warn", "deny", "expect", "forbid", "must_use", "doc",
        "inline", "path", "repr", "non_exhaustive", "rustfmt", "clippy", "diagnostic",
        "track_caller", "ignore", "should_panic",
    ];
    const KNOWN_MACROS: [&str; 33] = [
        "format", "write", "writeln", "print", "println", "eprint", "eprintln", "panic", "assert",
        "assert_eq", "assert_ne", "debug_assert", "debug_assert_eq", "debug_assert_ne",
        "format_args", "unreachable", "todo", "unimplemented", "vec", "matches", "concat",
        "stringify", "include_str", "include_bytes", "env", "option_env", "line", "column", "file",
        "module_path", "cfg", "compile_error", "dbg",
    ];
    const DERIVE: &str = "derive";

    pub fn of_impl(item: &ItemImpl, site: Site) -> Option<Door> {
        DoorKind::of_impl(item).map(|kind| Door {
            kind,
            target: Self::target_of(&item.self_ty),
            site: Some(site),
        })
    }

    pub fn of_callee(callee: &Ident, target: Option<Name>, within: Option<&Door>) -> Option<Door> {
        DoorKind::of_callee(callee)
            .filter(|_| {
                target
                    .as_ref()
                    .is_none_or(|name| !name.matches::<Whole>(&Self::STD_TYPES))
            })
            .map(|kind| Door {
                kind,
                target,
                site: Self::site_of(within),
            })
    }

    pub fn mentioned(names: Vec<Name>, within: Option<&Door>) -> Vec<Door> {
        names
            .into_iter()
            .filter(|name| !name.matches::<Whole>(&Self::STD_TYPES))
            .flat_map(|name| {
                DoorKind::EVERY.map(|kind| Door {
                    kind,
                    target: Some(name.clone()),
                    site: Self::site_of(within),
                })
            })
            .collect()
    }

    pub fn in_macro(invocation: &Macro, within: Option<&Door>) -> Vec<Door> {
        if Self::is_known_macro(invocation) {
            Self::sighted(DoorKind::in_known(invocation.tokens.clone()), within)
        } else {
            Vec::new()
        }
    }

    pub fn unknown_tokens(invocation: &Macro) -> TokenStream {
        if Self::is_known_macro(invocation) {
            TokenStream::new()
        } else {
            invocation.tokens.clone()
        }
    }

    pub fn signal_tokens(attribute: &Attribute) -> TokenStream {
        match &attribute.meta {
            Meta::List(list) if Self::is_signal(attribute) => list.tokens.clone(),
            _ => TokenStream::new(),
        }
    }

    pub fn in_unknown(tokens: &TokenStream, within: Option<&Door>) -> Vec<Door> {
        Self::sighted(DoorKind::in_unknown(tokens), within)
    }

    pub fn mentioned_in(tokens: TokenStream, within: Option<&Door>) -> Vec<Door> {
        Self::mentioned(Self::names_in_tokens(tokens), within)
    }

    pub fn of_type_item(
        attributes: &[&Attribute],
        names: Vec<Name>,
        within: Option<&Door>,
    ) -> Vec<Door> {
        if attributes
            .iter()
            .any(|attribute| Self::is_signal(attribute))
        {
            Self::mentioned(names, within)
        } else {
            Vec::new()
        }
    }

    pub fn is_known_macro(invocation: &Macro) -> bool {
        invocation
            .path
            .segments
            .last()
            .is_some_and(|segment| Name::from(&segment.ident).matches::<Whole>(&Self::KNOWN_MACROS))
    }

    pub fn is_signal(attribute: &Attribute) -> bool {
        let name = attribute
            .path()
            .segments
            .first()
            .map(|segment| Name::from(&segment.ident))
            .unwrap_or_default();
        match name {
            name if name == Self::DERIVE => attribute
                .parse_args_with(Punctuated::<syn::Path, Token![,]>::parse_terminated)
                .map_or(true, |derived| derived.iter().any(Self::is_crate_derive)),
            name => !name.matches::<Whole>(&Self::SILENT_ATTRIBUTES),
        }
    }

    pub fn is_crate_derive(path: &syn::Path) -> bool {
        path.segments
            .last()
            .is_none_or(|segment| !Name::from(&segment.ident).matches::<Whole>(&Self::STD_DERIVES))
    }

    pub fn names_in(ty: &Type) -> Vec<Name> {
        Self::names_in_tokens(ty.to_token_stream())
    }

    pub fn names_in_arguments(arguments: Option<&AngleBracketedGenericArguments>) -> Vec<Name> {
        Self::names_in_tokens(arguments.map(ToTokens::to_token_stream).unwrap_or_default())
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

    pub fn is_active_among(&self, open: &[Door]) -> bool {
        self.site.is_none() || open.iter().any(|door| door.site == self.site)
    }

    pub fn is_reached_by(&self, seen: &Door, doors: &[Door]) -> bool {
        self.kind == seen.kind
            && seen.site != self.site
            && seen.target.as_ref().is_none_or(|target| {
                self.target.as_ref().is_none_or(|own| own == target)
                    || !doors
                        .iter()
                        .any(|door| door.target.as_ref() == Some(target))
            })
    }

    pub fn is_mentioned_by(&self, seen: &Door) -> bool {
        self.kind == seen.kind
            && seen.site != self.site
            && (self.target.is_none() || seen.target == self.target)
    }

    fn sighted(kinds: Vec<DoorKind>, within: Option<&Door>) -> Vec<Door> {
        kinds
            .into_iter()
            .map(|kind| Door {
                kind,
                target: None,
                site: Self::site_of(within),
            })
            .collect()
    }

    fn names_in_tokens(tokens: TokenStream) -> Vec<Name> {
        tokens
            .into_iter()
            .flat_map(|tree| match tree {
                TokenTree::Ident(ident) => vec![Name::from(&ident)],
                TokenTree::Group(group) => Self::names_in_tokens(group.stream()),
                TokenTree::Literal(_) | TokenTree::Punct(_) => Vec::new(),
            })
            .collect()
    }

    fn site_of(within: Option<&Door>) -> Option<Site> {
        within.and_then(|door| door.site.clone())
    }
}
