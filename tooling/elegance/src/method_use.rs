use crate::method_name::MethodName;
use crate::receiver_kind::ReceiverKind;
use crate::type_key::TypeKey;
use crate::usage::Usage;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MethodUse {
    owner: TypeKey,
    name: MethodName,
    receiver: ReceiverKind,
    usage: Usage,
}

impl MethodUse {
    #[must_use]
    pub fn new(owner: TypeKey, name: MethodName, receiver: ReceiverKind, usage: Usage) -> MethodUse {
        MethodUse { owner, name, receiver, usage }
    }

    #[must_use]
    pub fn owner(&self) -> &TypeKey {
        &self.owner
    }

    #[must_use]
    pub fn name(&self) -> &MethodName {
        &self.name
    }

    #[must_use]
    pub fn receiver(&self) -> ReceiverKind {
        self.receiver
    }

    #[must_use]
    pub fn usage(&self) -> &Usage {
        &self.usage
    }
}
