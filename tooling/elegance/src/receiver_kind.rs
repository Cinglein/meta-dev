use syn::{Receiver, Type};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReceiverKind {
    Shared,
    Exclusive,
    Owned,
}

impl ReceiverKind {
    #[must_use]
    pub fn taken_by(receiver: &Receiver) -> ReceiverKind {
        match &*receiver.ty {
            Type::Reference(reference) if reference.mutability.is_some() => ReceiverKind::Exclusive,
            Type::Reference(_) => ReceiverKind::Shared,
            _ => ReceiverKind::Owned,
        }
    }
}
