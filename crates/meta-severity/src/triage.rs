use meta_schema::{ItemKind, LevelName, Placement};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Triage {
    placement: Placement,
    kind: ItemKind,
    level: Option<LevelName>,
}

impl Triage {
    #[must_use]
    pub fn new(placement: Placement, kind: ItemKind, level: Option<LevelName>) -> Triage {
        Triage { placement, kind, level }
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
    pub fn level(&self) -> Option<&LevelName> {
        self.level.as_ref()
    }
}
