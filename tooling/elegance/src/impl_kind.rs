use std::collections::BTreeSet;

use crate::trait_key::TraitKey;
use crate::type_key::TypeKey;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ImplKind {
    Inherent(TypeKey),
    Concrete { implemented: TraitKey, target: TypeKey },
    Blanket { implemented: TraitKey, bounds: BTreeSet<TraitKey> },
}

impl ImplKind {
    #[must_use]
    pub fn target(&self) -> Option<&TypeKey> {
        match self {
            ImplKind::Inherent(target) | ImplKind::Concrete { target, .. } => Some(target),
            ImplKind::Blanket { .. } => None,
        }
    }

    #[must_use]
    pub fn implemented(&self) -> Option<&TraitKey> {
        match self {
            ImplKind::Inherent(_) => None,
            ImplKind::Concrete { implemented, .. } | ImplKind::Blanket { implemented, .. } => Some(implemented),
        }
    }

    #[must_use]
    pub fn blanket_bounds(&self) -> Option<&BTreeSet<TraitKey>> {
        match self {
            ImplKind::Blanket { bounds, .. } => Some(bounds),
            ImplKind::Inherent(_) | ImplKind::Concrete { .. } => None,
        }
    }
}
