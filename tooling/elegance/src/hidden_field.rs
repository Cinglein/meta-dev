use std::collections::BTreeSet;
use std::fmt;

use crate::field::Field;
use crate::located::Located;
use crate::places::Places;
use crate::source_path::SourcePath;
use crate::type_key::TypeKey;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HiddenField {
    owner: TypeKey,
    field: Field,
}

impl HiddenField {
    #[must_use]
    pub fn new(owner: TypeKey, field: Field) -> HiddenField {
        HiddenField { owner, field }
    }
}

impl Located for HiddenField {
    fn places(&self, places: &Places) -> BTreeSet<SourcePath> {
        self.owner.places(places)
    }
}

impl fmt::Display for HiddenField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}.{} ({})", self.owner, self.field, self.field.kind())
    }
}
