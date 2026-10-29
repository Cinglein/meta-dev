use meta_schema::{ItemKind, Placement};

use crate::reading::Reading;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Triage {
    placement: Placement,
    kind: ItemKind,
    reading: Reading,
}

impl Triage {
    #[must_use]
    pub fn new(placement: Placement, kind: ItemKind, reading: Reading) -> Triage {
        Triage { placement, kind, reading }
    }

    #[must_use]
    pub fn placement(&self) -> &Placement {
        &self.placement
    }

    #[must_use]
    pub fn kind(&self) -> ItemKind {
        self.kind
    }

    #[must_use]
    pub fn reading(&self) -> &Reading {
        &self.reading
    }
}
