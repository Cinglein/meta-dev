use std::collections::{BTreeMap, BTreeSet};

use syn::visit::Visit;
use syn::{Fields, ItemEnum, ItemImpl, ItemType};

use super::type_name::TypeName;
use crate::count::Count;
use crate::rule::name::Name;
use crate::source_file::SourceFile;

#[derive(Default)]
pub struct Declarations {
    vertices: BTreeSet<TypeName>,
    aliases: BTreeMap<TypeName, TypeName>,
    sum_types: BTreeMap<Name, String>,
    path: String,
}

impl Declarations {
    pub fn collect(files: &[SourceFile]) -> Declarations {
        let declarations = files.iter().fold(Declarations::default(), |mut declarations, file| {
            file.path().clone_into(&mut declarations.path);
            declarations.visit_file(file.syntax());
            declarations
        });
        let resolved: BTreeSet<TypeName> =
            declarations.vertices.iter().map(|vertex| declarations.resolve(vertex.clone())).collect();
        Declarations { vertices: resolved, ..declarations }
    }

    pub fn resolve(&self, name: TypeName) -> TypeName {
        self.aliases.get(&name).cloned().unwrap_or(name)
    }

    pub fn is_vertex(&self, name: &TypeName) -> bool {
        self.vertices.contains(name)
    }

    pub fn vertex_count(&self) -> Count {
        Count::of_len(self.vertices.len())
    }

    pub fn is_foreign_sum_type(&self, enum_name: &Name, path: &str) -> bool {
        self.sum_types.get(enum_name).is_some_and(|home| home != path)
    }
}

impl<'ast> Visit<'ast> for Declarations {
    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        self.vertices.extend(
            item.trait_
                .as_ref()
                .and_then(|(_, path, _)| path.segments.last())
                .is_some_and(|segment| segment.ident == "State")
                .then(|| TypeName::from_type(&item.self_ty)),
        );
        syn::visit::visit_item_impl(self, item);
    }

    fn visit_item_type(&mut self, item: &'ast ItemType) {
        self.aliases.insert(TypeName::named(&item.ident), TypeName::from_type(&item.ty));
    }

    fn visit_item_enum(&mut self, item: &'ast ItemEnum) {
        self.sum_types.extend(
            item.variants
                .iter()
                .any(|variant| !matches!(variant.fields, Fields::Unit))
                .then(|| (Name::from(&item.ident), self.path.clone())),
        );
    }
}
