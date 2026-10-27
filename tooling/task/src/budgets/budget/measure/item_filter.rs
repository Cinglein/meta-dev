use meta_severity::SeverityFilter;
use serde::{Deserialize, Serialize};
use strum::Display;

use super::item_kind::ItemKind;
use crate::policy::Most;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Display, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ItemFilter {
    #[strum(to_string = "{0}s")]
    Kind(ItemKind),
    InDiscussion,
    UnheldTasks,
    #[strum(to_string = "{0}")]
    Severity(SeverityFilter),
    #[strum(to_string = "verbose {0}")]
    Verbose(Most),
}

impl ItemFilter {
    #[must_use]
    pub const fn severity(&self) -> Option<&SeverityFilter> {
        match self {
            ItemFilter::Severity(severity) => Some(severity),
            ItemFilter::Kind(_)
            | ItemFilter::InDiscussion
            | ItemFilter::UnheldTasks
            | ItemFilter::Verbose(_) => None,
        }
    }

    #[must_use]
    pub fn picked(&self, marks: &[ItemFilter], words: Most) -> Option<&ItemFilter> {
        let picked = match self {
            ItemFilter::Verbose(most) => words > *most,
            ItemFilter::Kind(_)
            | ItemFilter::InDiscussion
            | ItemFilter::UnheldTasks
            | ItemFilter::Severity(_) => marks.contains(self),
        };
        picked.then_some(self)
    }
}
