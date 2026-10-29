use syn::Receiver;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReceiverKind {
    Shared,
    Exclusive,
    Owned,
}

impl ReceiverKind {
    #[must_use]
    pub fn taken_by(receiver: &Receiver) -> ReceiverKind {
        match (&receiver.reference, &receiver.mutability) {
            (Some(_), Some(_)) => ReceiverKind::Exclusive,
            (Some(_), None) => ReceiverKind::Shared,
            (None, _) => ReceiverKind::Owned,
        }
    }
}
