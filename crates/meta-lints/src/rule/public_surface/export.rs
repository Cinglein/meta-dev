use std::fmt;

use syn::{Item, ItemUse, UseTree, Visibility};

use crate::line_number::LineNumber;
use crate::rule::binary_files::ModuleLocation;
use crate::rule::name::Name;
use crate::source_file::SourceFile;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Export {
    crate_name: String,
    name: Name,
    line: LineNumber,
}

impl Export {
    pub fn in_root(file: &SourceFile) -> Vec<Export> {
        file.crate_name()
            .filter(|_| ModuleLocation::from(file).is_library_root())
            .map(|crate_name| Self::public_uses(file).flat_map(|used| Self::leaves(&crate_name, &used.tree)).collect())
            .unwrap_or_default()
    }

    pub fn crate_name(&self) -> &str {
        &self.crate_name
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn line(&self) -> LineNumber {
        self.line
    }

    fn public_uses(file: &SourceFile) -> impl Iterator<Item = &ItemUse> {
        file.syntax().items.iter().filter_map(|item| match item {
            Item::Use(used) if !matches!(used.vis, Visibility::Inherited) => Some(used),
            _ => None,
        })
    }

    fn leaves(crate_name: &str, tree: &UseTree) -> Vec<Export> {
        match tree {
            UseTree::Path(path) => Self::leaves(crate_name, &path.tree),
            UseTree::Group(group) => group.items.iter().flat_map(|item| Self::leaves(crate_name, item)).collect(),
            UseTree::Name(name) => vec![Export {
                crate_name: crate_name.to_owned(),
                name: Name::from(&name.ident),
                line: LineNumber::starting(name.ident.span()),
            }],
            UseTree::Rename(rename) => vec![Export {
                crate_name: crate_name.to_owned(),
                name: Name::from(&rename.rename),
                line: LineNumber::starting(rename.rename.span()),
            }],
            UseTree::Glob(_) => Vec::new(),
        }
    }
}

impl fmt::Display for Export {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} is exported but no other crate names it; export it when a user appears", self.name)
    }
}
