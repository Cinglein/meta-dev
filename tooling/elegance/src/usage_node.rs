use crate::field_name::FieldName;
use crate::method_key::MethodKey;
use crate::method_name::MethodName;
use crate::resolution::Resolution;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum UsageNode {
    Field(FieldName),
    Call(MethodName),
    Own(MethodKey),
    Whole,
}

impl UsageNode {
    #[must_use]
    pub fn own(self) -> Option<MethodKey> {
        match self {
            UsageNode::Own(key) => Some(key),
            UsageNode::Field(_) | UsageNode::Call(_) | UsageNode::Whole => None,
        }
    }

    #[must_use]
    pub fn resolved(self, resolution: &Resolution) -> Vec<UsageNode> {
        match self {
            UsageNode::Whole => resolution.fields(),
            UsageNode::Call(name) => resolution.called(&name),
            UsageNode::Field(_) | UsageNode::Own(_) => vec![self],
        }
    }
}
