use std::iter;

use proc_macro2::{Delimiter, Ident, TokenStream, TokenTree};

use super::finding::Finding;
use task::LineNumber;
use task::Site;
use task::SourceFile;
use task::Violation;
use task::{Casing, Name, Whole};

pub struct FileSites;

impl FileSites {
    const OWNERS: [&str; 3] = ["std", "tokio", "unix"];
    const MODULE: &str = "fs";
    const FILE: &str = "File";
    const OPENING: [&str; 3] = ["open", "create", "options"];
    const OPTIONS: &str = "OpenOptions";
    const PATH_SEPARATOR: char = ':';
    const LIST_SEPARATOR: char = ',';

    pub fn violations(file: &SourceFile) -> Vec<Violation> {
        file.text()
            .parse::<TokenStream>()
            .map(Self::reaching)
            .unwrap_or_default()
            .into_iter()
            .map(|access| {
                Violation::new(
                    Site::Line(file.path().into(), LineNumber::from(access.span())),
                    Finding::FileOutsideDoor {
                        access: Name::from(&access),
                    },
                )
            })
            .collect()
    }

    fn reaching(stream: TokenStream) -> Vec<Ident> {
        let trees: Vec<TokenTree> = stream.into_iter().collect();
        let nested = trees.iter().flat_map(|tree| match tree {
            TokenTree::Group(group) => Self::reaching(group.stream()),
            _ => Vec::new(),
        });
        iter::successors(Some(trees.as_slice()), |rest| {
            rest.split_first().map(|(_, rest)| rest)
        })
        .filter_map(Self::access)
        .chain(nested)
        .collect()
    }

    fn access(trees: &[TokenTree]) -> Option<Ident> {
        match trees {
            [
                TokenTree::Ident(owner),
                TokenTree::Punct(first),
                TokenTree::Punct(second),
                rest @ ..,
            ] if first.as_char() == Self::PATH_SEPARATOR
                && second.as_char() == Self::PATH_SEPARATOR =>
            {
                Self::reached(owner, rest)
            }
            _ => None,
        }
    }

    fn reached(owner: &Ident, rest: &[TokenTree]) -> Option<Ident> {
        let owns = Name::from(owner).matches::<Whole>(&Self::OWNERS);
        let module = |item: &[TokenTree]| match item {
            [TokenTree::Ident(module), after @ ..]
                if *module == Self::MODULE && Self::opens(after) =>
            {
                Some(module.clone())
            }
            _ => None,
        };
        match rest {
            [TokenTree::Ident(opening), ..]
                if *owner == Self::FILE && Name::from(opening).matches::<Whole>(&Self::OPENING) =>
            {
                Some(opening.clone())
            }
            [TokenTree::Group(group), ..] if owns && group.delimiter() == Delimiter::Brace => {
                let items: Vec<TokenTree> = group.stream().into_iter().collect();
                items
                    .split(|tree| matches!(tree, TokenTree::Punct(comma) if comma.as_char() == Self::LIST_SEPARATOR))
                    .find_map(module)
            }
            _ if owns => module(rest),
            _ => None,
        }
    }

    fn opens(after: &[TokenTree]) -> bool {
        let opening = |tree: &TokenTree| {
            matches!(tree, TokenTree::Ident(name)
                if *name == Self::OPTIONS || Name::from(name).casing() == Casing::Lower)
        };
        let spread = |tree: &TokenTree| match tree {
            TokenTree::Group(group) => group.stream().into_iter().collect(),
            other => vec![other.clone()],
        };
        match after {
            [TokenTree::Punct(first), TokenTree::Punct(_), next, ..]
                if first.as_char() == Self::PATH_SEPARATOR =>
            {
                spread(next).iter().any(opening)
            }
            _ => true,
        }
    }
}
