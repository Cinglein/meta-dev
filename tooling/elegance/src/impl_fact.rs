use crate::impl_kind::ImplKind;
use crate::signature::Signature;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImplFact {
    kind: ImplKind,
    bodies: Vec<Signature>,
}

impl ImplFact {
    #[must_use]
    pub fn new(kind: ImplKind, bodies: Vec<Signature>) -> ImplFact {
        ImplFact { kind, bodies }
    }

    #[must_use]
    pub fn kind(&self) -> &ImplKind {
        &self.kind
    }

    #[must_use]
    pub fn bodies(&self) -> &[Signature] {
        &self.bodies
    }
}
