use std::collections::BTreeSet;

use syn::Ident;
use syn::visit::Visit;

use crate::rule::name::Name;
use crate::source_file::SourceFile;

pub struct NameUses {
    path: String,
    names: BTreeSet<Name>,
}

impl NameUses {
    pub fn in_file(file: &SourceFile) -> NameUses {
        let mut uses = NameUses { path: file.path().to_owned(), names: BTreeSet::new() };
        uses.visit_file(file.syntax());
        uses
    }

    pub fn mentions(&self, name: &Name) -> bool {
        self.names.contains(name)
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}

impl<'ast> Visit<'ast> for NameUses {
    fn visit_ident(&mut self, ident: &'ast Ident) {
        self.names.insert(Name::from(ident));
    }
}
