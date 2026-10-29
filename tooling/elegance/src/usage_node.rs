use crate::field_name::FieldName;
use crate::method_name::MethodName;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum UsageNode {
    Field(FieldName),
    Method(MethodName),
    Whole,
}

impl UsageNode {
    #[must_use]
    pub fn method(self) -> Option<MethodName> {
        match self {
            UsageNode::Method(name) => Some(name),
            UsageNode::Field(_) | UsageNode::Whole => None,
        }
    }

    #[must_use]
    pub fn field(&self) -> Option<&FieldName> {
        match self {
            UsageNode::Field(name) => Some(name),
            UsageNode::Method(_) | UsageNode::Whole => None,
        }
    }
}
