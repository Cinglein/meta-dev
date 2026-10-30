use crate::field_name::FieldName;
use crate::write_kind::WriteKind;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct FieldWrite {
    field: FieldName,
    how: WriteKind,
}

impl FieldWrite {
    #[must_use]
    pub fn new(field: FieldName, how: WriteKind) -> FieldWrite {
        FieldWrite { field, how }
    }

    #[must_use]
    pub fn field(&self) -> &FieldName {
        &self.field
    }

    #[must_use]
    pub fn how(&self) -> WriteKind {
        self.how
    }
}
