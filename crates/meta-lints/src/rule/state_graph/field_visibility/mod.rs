mod public_field;

use public_field::PublicField;
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{ItemStruct, Visibility};

use crate::site::Site;
use crate::source_file::SourceFile;
use crate::violation::Violation;

pub struct FieldVisibility<'scan> {
    path: &'scan str,
    violations: Vec<Violation>,
}

impl<'scan> FieldVisibility<'scan> {
    pub fn violations(file: &'scan SourceFile) -> Vec<Violation> {
        let mut fields = FieldVisibility { path: file.path(), violations: Vec::new() };
        fields.visit_file(file.syntax());
        fields.violations
    }
}

impl<'ast> Visit<'ast> for FieldVisibility<'_> {
    fn visit_item_struct(&mut self, item: &'ast ItemStruct) {
        let exposed = item
            .fields
            .iter()
            .enumerate()
            .filter(|(_, field)| !matches!(field.vis, Visibility::Inherited))
            .map(|(index, field)| {
                Violation::new(Site::spanned(self.path, field.span()), PublicField::at_index(field, index, &item.ident))
            });
        self.violations.extend(exposed);
    }
}
