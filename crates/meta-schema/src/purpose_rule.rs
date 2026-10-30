use thiserror::Error;

use crate::purpose::Purpose;
use crate::schema_version::SchemaVersion;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PurposeRule {
    Optional,
    Required,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
#[error("a task needs a purpose")]
pub struct PurposeMissing;

impl PurposeRule {
    #[must_use]
    pub fn under(marked: Option<SchemaVersion>) -> Self {
        marked.map_or(Self::Optional, SchemaVersion::purpose_rule)
    }

    pub fn admitting(self, purpose: Option<Purpose>) -> Result<(), PurposeMissing> {
        match (self, purpose) {
            (Self::Required, None) => Err(PurposeMissing),
            (Self::Optional, _) | (Self::Required, Some(_)) => Ok(()),
        }
    }
}
