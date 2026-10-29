use super::invocation::Invocation;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Starter {
    Tests,
    Meta(Invocation),
}

impl Starter {
    pub(crate) fn command(&self) -> Option<&str> {
        match self {
            Starter::Tests => None,
            Starter::Meta(invocation) => Some(invocation.command()),
        }
    }
}
