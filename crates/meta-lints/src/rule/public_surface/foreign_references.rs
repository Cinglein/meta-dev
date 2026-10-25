use std::collections::{BTreeMap, BTreeSet};

use syn::visit::Visit;
use syn::{ItemUse, Path, UseTree};

use super::export::Export;
use crate::rule::binary_files::BinaryFiles;
use crate::rule::name::Name;
use crate::source_file::SourceFile;

#[derive(Default)]
pub struct ForeignReferences {
    own_crate: String,
    referenced: BTreeMap<Name, BTreeSet<Name>>,
}

impl ForeignReferences {
    pub fn in_file(file: &SourceFile, binaries: &BinaryFiles) -> ForeignReferences {
        let own_crate = file.crate_name().filter(|_| !binaries.contains(file.path())).unwrap_or_default();
        let mut references = ForeignReferences { own_crate, referenced: BTreeMap::new() };
        references.visit_file(file.syntax());
        references
    }

    pub fn absorb(self, other: ForeignReferences) -> ForeignReferences {
        let referenced = other.referenced.into_iter().fold(self.referenced, |mut referenced, (crate_name, names)| {
            referenced.entry(crate_name).or_default().extend(names);
            referenced
        });
        ForeignReferences { own_crate: self.own_crate, referenced }
    }

    pub fn depends_on(&self, crate_name: &str) -> bool {
        self.referenced.keys().any(|referenced| referenced == crate_name)
    }

    pub fn names(&self, export: &Export) -> bool {
        self.referenced
            .iter()
            .any(|(crate_name, names)| crate_name == export.crate_name() && names.contains(export.name()))
    }

    fn walk(&mut self, first: Option<&Name>, tree: &UseTree) {
        match tree {
            UseTree::Path(path) => {
                let ident = Name::from(&path.ident);
                self.walk(Some(first.unwrap_or(&ident)), &path.tree);
            }
            UseTree::Group(group) => group.items.iter().for_each(|item| self.walk(first, item)),
            UseTree::Name(name) => self.record(first, &Name::from(&name.ident)),
            UseTree::Rename(rename) => self.record(first, &Name::from(&rename.ident)),
            UseTree::Glob(_) => {}
        }
    }

    fn record(&mut self, first: Option<&Name>, name: &Name) {
        first.filter(|crate_name| crate_name.as_str() != self.own_crate).into_iter().for_each(|crate_name| {
            self.referenced.entry(crate_name.clone()).or_default().insert(name.clone());
        });
    }
}

impl<'ast> Visit<'ast> for ForeignReferences {
    fn visit_item_use(&mut self, item: &'ast ItemUse) {
        self.walk(None, &item.tree);
    }

    fn visit_path(&mut self, path: &'ast Path) {
        let segments: Vec<Name> = path.segments.iter().map(|segment| Name::from(&segment.ident)).collect();
        segments.first().zip(segments.get(1)).into_iter().for_each(|(first, second)| self.record(Some(first), second));
        syn::visit::visit_path(self, path);
    }
}
