use std::collections::BTreeSet;

use syn::visit::Visit;
use syn::{ItemEnum, ItemStruct, ItemTrait};

use task::Name;
use task::SourceFile;

#[derive(Default)]
pub struct WorkspaceTypes(BTreeSet<Name>);

impl WorkspaceTypes {
    pub fn collect(files: &[SourceFile]) -> WorkspaceTypes {
        files
            .iter()
            .fold(WorkspaceTypes::default(), |mut types, file| {
                types.visit_file(file.syntax());
                types
            })
    }

    pub fn declares(&self, name: &Name) -> bool {
        self.0.contains(name)
    }
}

impl<'ast> Visit<'ast> for WorkspaceTypes {
    fn visit_item_struct(&mut self, item: &'ast ItemStruct) {
        self.0.insert(Name::from(&item.ident));
    }

    fn visit_item_enum(&mut self, item: &'ast ItemEnum) {
        self.0.insert(Name::from(&item.ident));
    }

    fn visit_item_trait(&mut self, item: &'ast ItemTrait) {
        self.0.insert(Name::from(&item.ident));
    }
}
