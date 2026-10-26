use serde::Deserialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(from = "bool")]
pub enum Requirement {
    Required,
    Optional,
}

impl From<bool> for Requirement {
    fn from(required: bool) -> Self {
        if required { Self::Required } else { Self::Optional }
    }
}
