use std::path::PathBuf;

use syn::visit::Visit;
use syn::{ItemImpl, ItemStruct, Type};

use super::deserialized_struct::DeserializedStruct;
use task::Name;
use task::SourceFile;

pub struct SchemaScan {
    path: PathBuf,
    structs: Vec<DeserializedStruct>,
    seeds: Vec<Name>,
}

impl SchemaScan {
    const OWNED_SCHEMA: &str = "EmbeddedDefaults";

    pub fn in_file(file: &SourceFile) -> SchemaScan {
        let mut scan = SchemaScan {
            path: file.path().to_owned(),
            structs: Vec::new(),
            seeds: Vec::new(),
        };
        scan.visit_file(file.syntax());
        scan
    }

    pub fn structs(&self) -> impl Iterator<Item = &DeserializedStruct> {
        self.structs.iter()
    }

    pub fn seeds(&self) -> impl Iterator<Item = &Name> {
        self.seeds.iter()
    }

    fn implements_owned_schema(item: &ItemImpl) -> Option<Name> {
        let implemented = item
            .trait_
            .as_ref()
            .and_then(|(_, path, _)| path.segments.last())?;
        match &*item.self_ty {
            Type::Path(self_type) => (implemented.ident == Self::OWNED_SCHEMA)
                .then_some(self_type)
                .and_then(|self_type| self_type.path.segments.last())
                .map(|segment| Name::from(&segment.ident)),
            _ => None,
        }
    }
}

impl<'ast> Visit<'ast> for SchemaScan {
    fn visit_item_struct(&mut self, item: &'ast ItemStruct) {
        self.structs
            .extend(DeserializedStruct::from_item(&self.path, item));
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        self.seeds.extend(Self::implements_owned_schema(item));
        syn::visit::visit_item_impl(self, item);
    }
}
