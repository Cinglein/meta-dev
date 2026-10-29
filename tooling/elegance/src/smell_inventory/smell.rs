mod confidence;
mod smell_location;
mod smell_tag;

use std::collections::BTreeSet;

use serde::de::IgnoredAny;
use serde::{Deserialize, Serialize};

use super::smell_id::SmellId;
use crate::source_path::SourcePath;
use confidence::Confidence;
use smell_location::SmellLocation;
use smell_tag::SmellTag;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Smell {
    id: SmellId,
    #[serde(skip_serializing)]
    category: IgnoredAny,
    locations: Vec<SmellLocation>,
    #[serde(skip_serializing)]
    why: IgnoredAny,
    confidence: Confidence,
    measure: SmellTag,
}

impl Smell {
    #[must_use]
    pub fn smell_id(&self) -> &SmellId {
        &self.id
    }

    #[must_use]
    pub fn is_tagged(&self, tag: &str) -> bool {
        self.measure == *tag
    }

    #[must_use]
    pub fn is_flagged_by(&self, spots: &BTreeSet<SourcePath>) -> bool {
        self.locations.iter().any(|location| spots.contains(&location.file()))
    }
}
