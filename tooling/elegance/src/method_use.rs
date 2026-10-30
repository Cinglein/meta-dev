use crate::field_name::FieldName;
use crate::method_key::MethodKey;
use crate::receiver_kind::ReceiverKind;
use crate::type_key::TypeKey;
use crate::usage::Usage;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MethodUse {
    owner: TypeKey,
    key: MethodKey,
    receiver: ReceiverKind,
    usage: Usage,
    accessed: Option<FieldName>,
}

impl MethodUse {
    #[must_use]
    pub fn new(owner: TypeKey, key: MethodKey, receiver: ReceiverKind, usage: Usage) -> MethodUse {
        MethodUse { owner, key, receiver, usage, accessed: None }
    }

    #[must_use]
    pub fn accessing(self, accessed: Option<FieldName>) -> MethodUse {
        MethodUse { accessed, ..self }
    }

    #[must_use]
    pub fn mutates(&self) -> bool {
        self.receiver == ReceiverKind::Exclusive || !self.usage.writes().is_empty()
    }

    #[must_use]
    pub fn owner(&self) -> &TypeKey {
        &self.owner
    }

    #[must_use]
    pub fn key(&self) -> &MethodKey {
        &self.key
    }

    #[must_use]
    pub fn receiver(&self) -> ReceiverKind {
        self.receiver
    }

    #[must_use]
    pub fn usage(&self) -> &Usage {
        &self.usage
    }

    #[must_use]
    pub fn accessed(&self) -> Option<&FieldName> {
        self.accessed.as_ref()
    }
}
