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
    const PATHS: [&str; 2] = ["Path", "PathBuf"];
    const QUERIES: [&str; 9] = [
        "exists", "try_exists", "is_file", "is_dir", "metadata", "symlink_metadata",
        "canonicalize", "read_dir", "read_link",
    ];
    const MODULE: &str = "fs";
    const FILE: &str = "File";
    const PATH_SEPARATOR: char = ':';
    const LIST_SEPARATOR: char = ',';
    const METHOD: char = '.';

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
                TokenTree::Punct(dot),
                TokenTree::Ident(query),
                TokenTree::Group(arguments),
                ..,
            ] if dot.as_char() == Self::METHOD
                && arguments.delimiter() == Delimiter::Parenthesis
                && Name::from(query).matches::<Whole>(&Self::QUERIES) =>
            {
                Some(query.clone())
            }
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
            [TokenTree::Ident(module), ..] if *module == Self::MODULE => Some(module.clone()),
            _ => None,
        };
        match rest {
            [TokenTree::Ident(opening), ..]
                if *owner == Self::FILE && Name::from(opening).casing() == Casing::Lower =>
            {
                Some(opening.clone())
            }
            [TokenTree::Ident(query), ..]
                if Name::from(owner).matches::<Whole>(&Self::PATHS)
                    && Name::from(query).matches::<Whole>(&Self::QUERIES) =>
            {
                Some(query.clone())
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
}
