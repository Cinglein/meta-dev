use syn::{Fields, Index, Member};

use super::super::declarations::Declarations;
use crate::field::Field;
use crate::field_name::FieldName;
use crate::layout::Layout;

#[derive(Debug)]
pub struct LayoutScan<'declared> {
    declarations: &'declared Declarations,
}

impl<'declared> LayoutScan<'declared> {
    #[must_use]
    pub fn new(declarations: &'declared Declarations) -> LayoutScan<'declared> {
        LayoutScan { declarations }
    }

    #[must_use]
    pub fn scanned(&self, fields: &Fields) -> Layout {
        let positional = fields
            .iter()
            .enumerate()
            .filter(|(_, field)| field.ident.is_none())
            .map(|(position, field)| self.field(&Member::Unnamed(Index::from(position)), field))
            .collect();
        let named = fields
            .iter()
            .filter_map(|field| field.ident.clone().map(|ident| self.field(&Member::Named(ident), field)))
            .collect();
        Layout::new(named, positional)
    }

    #[must_use]
    pub fn field(&self, member: &Member, field: &syn::Field) -> Field {
        Field::new(FieldName::new(member), &field.ty, self.declarations.field_kind(&field.ty))
    }
}
