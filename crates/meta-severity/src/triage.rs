use meta_schema::{ItemKind, Placement};

use crate::promoted::Promoted;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Triage {
    placement: Placement,
    kind: ItemKind,
    promoted: Promoted,
}

impl Triage {
    #[must_use]
    pub fn new(placement: Placement, kind: ItemKind, promoted: Promoted) -> Triage {
        Triage { placement, kind, promoted }
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
    pub fn promoted(&self) -> &Promoted {
        &self.promoted
    }
}
