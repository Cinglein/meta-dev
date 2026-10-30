use std::fmt;

use syn::Type;

use crate::field_kind::FieldKind;
use crate::field_name::FieldName;
use crate::signature::Signature;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Field {
    name: FieldName,
    ty: String,
    kind: FieldKind,
}

impl Field {
    #[must_use]
    pub fn new(name: FieldName, ty: &Type, kind: FieldKind) -> Field {
        Field { name, ty: Signature::rendered(ty), kind }
    }

    #[must_use]
    pub fn name(&self) -> &FieldName {
        &self.name
    }

    #[must_use]
    pub fn kind(&self) -> FieldKind {
        self.kind
    }
}

impl fmt::Display for Field {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.name, self.ty)
    }
}
