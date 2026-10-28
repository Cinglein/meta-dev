use proc_macro2::{Delimiter, Ident, TokenStream, TokenTree};

use super::finding::Finding;
use task::LineNumber;
use task::Site;
use task::SourceFile;
use task::Violation;
use task::{Name, Whole};

pub struct FileSites;

impl FileSites {
    const OWNERS: [&str; 3] = ["std", "tokio", "unix"];
    const MODULE: &str = "fs";
    const FILE: &str = "File";
    const OPENING: [&str; 3] = ["open", "create", "options"];
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
        trees
            .windows(4)
            .filter_map(Self::access)
            .chain(nested)
            .collect()
    }

    fn access(window: &[TokenTree]) -> Option<Ident> {
        match window {
            [
                TokenTree::Ident(owner),
                TokenTree::Punct(first),
                TokenTree::Punct(second),
                reached,
            ] if first.as_char() == Self::PATH_SEPARATOR
                && second.as_char() == Self::PATH_SEPARATOR =>
            {
                Self::reached(owner, reached)
            }
            _ => None,
        }
    }

    fn reached(owner: &Ident, reached: &TokenTree) -> Option<Ident> {
        let owns = Name::from(owner).matches::<Whole>(&Self::OWNERS);
        match reached {
            TokenTree::Ident(module) if owns && *module == Self::MODULE => Some(module.clone()),
            TokenTree::Ident(opening)
                if *owner == Self::FILE && Name::from(opening).matches::<Whole>(&Self::OPENING) =>
            {
                Some(opening.clone())
            }
            TokenTree::Group(group) if owns && group.delimiter() == Delimiter::Brace => {
                let trees: Vec<TokenTree> = group.stream().into_iter().collect();
                trees
                    .split(|tree| matches!(tree, TokenTree::Punct(comma) if comma.as_char() == Self::LIST_SEPARATOR))
                    .find_map(|item| match item.first() {
                        Some(TokenTree::Ident(module)) if *module == Self::MODULE => Some(module.clone()),
                        _ => None,
                    })
            }
            _ => None,
        }
    }
}
